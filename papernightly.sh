rm nohup.out || true
nohup bash infra/localnightly.sh benchmarks/passing/ --paper & disown
