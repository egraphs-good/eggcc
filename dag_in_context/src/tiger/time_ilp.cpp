#include "time_ilp.h"

#include <algorithm>
#include <atomic>
#include <chrono>
#include <fstream>
#include <thread>
#include <unordered_set>
#include <iostream>

#include "greedy.h"
#include "ilp.h"
#include "regionalize.h"
#include "tiger.h"

using namespace std;

namespace {
using Clock = chrono::steady_clock;


void compute_tiger_metrics(ExtractRegionTiming &sample, const EGraph &gr,
                           EClassId root,
                           const vector<vector<Cost>> &rstatewalk_cost) {
  auto tiger_start = Clock::now();
  Extraction tiger_extraction =
      extract_regionalized_egraph_tiger(gr, root, rstatewalk_cost);
  (void)tiger_extraction;
  auto tiger_end = Clock::now();

  sample.tiger_duration_ns =
      chrono::duration_cast<chrono::nanoseconds>(tiger_end - tiger_start)
          .count();

  pair<StatewalkWidthReport, StatewalkWidthReport> res =
      get_stat_regionalized_egraph_tiger(gr, root, rstatewalk_cost);
  sample.statewalk_width_liveon_max = res.first.max_width;
  sample.statewalk_width_liveon_avg = res.first.avg_width;
  sample.statewalk_width_liveoff_max = res.second.max_width;
  sample.statewalk_width_liveoff_avg = res.second.avg_width;
}

void compute_ilp_metrics(ExtractRegionTiming &sample, const EGraph &gr,
                         EClassId root) {
  Extraction ilp_extraction;
  bool ilp_timed_out = false;
  bool ilp_infeasible = false;
  long long ilp_ns;
  ilp_ns = extract_region_ilp_with_timing(gr, root, ilp_extraction,
                                            ilp_timed_out, ilp_infeasible);

  sample.ilp_timed_out = ilp_timed_out;
  sample.ilp_infeasible = ilp_infeasible;
  if (ilp_timed_out || ilp_infeasible) {
    sample.ilp_duration_ns = nullopt;
  } else {
    sample.ilp_duration_ns = ilp_ns;
  }
}

ExtractRegionTiming
measure_region_timing(const EGraph &g, EClassId region_root,
                      const vector<vector<Cost>> &statewalk_cost) {
  const auto regionalized = construct_regionalized_egraph(g, region_root);

  const EGraph &gr = regionalized.first;
  const EClassId root = regionalized.second.first;
  const EGraphMapping &gr2g = regionalized.second.second;
  const vector<vector<Cost>> rstatewalk_cost =
      project_statewalk_cost(gr2g, statewalk_cost);

  ExtractRegionTiming sample;
  sample.egraph_size = g.eclasses.size();
  compute_tiger_metrics(sample, gr, root, rstatewalk_cost);
  compute_ilp_metrics(sample, gr, root);

  return sample;
}
} // namespace

vector<ExtractRegionTiming>
compute_extract_region_timings(const EGraph &g,
                               const vector<EClassId> &fun_roots) {
  vector<EClassId> region_roots = find_all_region_roots(g, fun_roots);

  vector<vector<Cost>> statewalk_cost = compute_statewalk_cost(g);

  vector<ExtractRegionTiming> timings(region_roots.size());
  if (region_roots.empty()) {
    return timings;
  }

  struct PreparedRegion {
    EGraph egraph;
    EClassId root;
    size_t index;
    vector<vector<Cost>> statewalk_cost;
  };

  vector<PreparedRegion> prepared_regions;
  prepared_regions.reserve(region_roots.size());

  for (size_t idx = 0; idx < region_roots.size(); ++idx) {
    auto regionalized = construct_regionalized_egraph(g, region_roots[idx]);
    PreparedRegion prepared{};
    prepared.egraph = std::move(regionalized.first);
    prepared.root = regionalized.second.first;
    prepared.index = idx;

    const EGraphMapping &gr2g = regionalized.second.second;
    prepared.statewalk_cost = project_statewalk_cost(gr2g, statewalk_cost);

    ExtractRegionTiming sample;
    sample.egraph_size = g.eclasses.size();
    compute_tiger_metrics(sample, prepared.egraph, prepared.root,
                          prepared.statewalk_cost);
    timings[idx] = sample;

    prepared_regions.push_back(std::move(prepared));
  }

  unsigned int hardware_threads = std::thread::hardware_concurrency();
  unsigned int usable_threads = hardware_threads == 0 ? 1 : hardware_threads;
  if (usable_threads > 30) {
    // leave one core free
    usable_threads = max(usable_threads - 1, 1u);
  }

  // divide by 4 because we spin up 4 benchmarks at once in profile.py
  if (usable_threads >= 7) {
    usable_threads /= 4;
  }

  
  size_t worker_count = min<size_t>(usable_threads, prepared_regions.size());
  if (worker_count == 0) {
    worker_count = 1;
  }

  std::atomic<size_t> next_index{0};

  cerr << "Running ILP timing on regions, one dot per region:";
  auto worker = [&]() {
    while (true) {
      size_t idx = next_index.fetch_add(1, std::memory_order_relaxed);
      if (idx >= prepared_regions.size()) {
        break;
      }
      cerr << ".";
      cerr.flush();
      const PreparedRegion &prepared = prepared_regions[idx];
      ExtractRegionTiming &sample = timings[prepared.index];
      compute_ilp_metrics(sample, prepared.egraph, prepared.root);
    }
  };
  cerr << "\n";

  vector<std::thread> threads;
  threads.reserve(worker_count);
  for (size_t i = 0; i < worker_count; ++i) {
    threads.emplace_back(worker);
  }
  for (auto &thread : threads) {
    thread.join();
  }

  return timings;
}

bool write_extract_region_timings_json(
    const vector<ExtractRegionTiming> &timings, const string &path) {
  ofstream out(path.c_str());
  if (!out.good()) {
    return false;
  }

  out << "{\n  \"rows\": [";
  if (!timings.empty()) {
    for (size_t i = 0; i < timings.size(); ++i) {
      const auto &sample = timings[i];
      out << (i == 0 ? "\n" : ",\n");
      out << "    {\"egraph_size\": " << sample.egraph_size
          << ", \"tiger_duration_ns\": " << sample.tiger_duration_ns
          << ", \"ilp_duration_ns\": ";
      if (sample.ilp_duration_ns.has_value()) {
        out << sample.ilp_duration_ns.value();
      } else {
        out << "null";
      }
      out << ", \"ilp_timed_out\": "
          << (sample.ilp_timed_out ? "true" : "false")
          << ", \"ilp_infeasible\": "
          << (sample.ilp_infeasible ? "true" : "false")
          << ", \"statewalk_width_liveon_max\": "
          << sample.statewalk_width_liveon_max
          << ", \"statewalk_width_liveon_avg\": "
          << sample.statewalk_width_liveon_avg
          << ", \"statewalk_width_liveoff_max\": "
          << sample.statewalk_width_liveoff_max
          << ", \"statewalk_width_liveoff_avg\": "
          << sample.statewalk_width_liveoff_avg << "}";
    }
    out << "\n  ";
  }
  out << "]\n}\n";
  return true;
}
