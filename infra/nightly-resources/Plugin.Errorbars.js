(function (global, factory) {
  typeof exports === "object" && typeof module !== "undefined"
    ? (module.exports = factory(require("chart.js")))
    : typeof define === "function" && define.amd
      ? define(["chart.js"], factory)
      : ((global = global || self),
        (global.PluginErrorbars = factory(global.Chart)));
})(this, function (Chart) {
  "use strict";

  Chart = Chart && Chart.hasOwnProperty("default") ? Chart["default"] : Chart;

  var defaultOptions = {
    /**
     * stroke color
     * @default: derived from borderColor
     */
    color: undefined,

    /**
     * width as number, or as string with pixel (px) ending, or as string with percentage (%) ending
     */
    width: 10,

    /**
     * lineWidth as number, or as string with pixel (px) ending, or array of such definition
     */
    lineWidth: 2,

    /**
     * whether the error values are given in absolute values or relative (default)
     */
    absoluteValues: false,
  };
  var ErrorBarsPlugin = {
    id: "chartJsPluginErrorBars",

    /**
     * get original barchart base bar coords
     * @param chart chartjs instance
     * @returns {Array} containing label, x, y and color
     * @private
     */
    _getBarchartBaseCoords: function _getBarchartBaseCoords(chart) {
      var coords = [];
      chart.data.datasets.forEach(function (d, i) {
        var bars = chart.getDatasetMeta(i).data;
        var values = d.data;
        coords.push(
          bars.map(function (b, j) {
            // line charts do not have labels in their meta data, access global label array instead
            var barLabel = "";

            if (!b._model.label) {
              barLabel = chart.data.labels[j];
            } else {
              barLabel = b._model.label; // required for hierarchical
            }

            return {
              label: barLabel,
              value: values[j],
              x: b._model.x,
              y: b._model.y,
              color: b._model.borderColor,
            };
          }),
        );
      });
      return coords;
    },

    /**
     * check whether the chart orientation is horizontal
     * @param chart chartjs instance
     * @returns {boolean}
     * @private
     */
    _isHorizontal: function _isHorizontal(chart) {
      return chart.config.type === "horizontalBar";
    },

    /**
     * compute error bars width in pixel or percent
     * @param chart chartjs instance
     * @param horizontal orientation
     * @param width plugin option width
     * @returns {*} width in pixel as number
     * @private
     */
    _computeWidth: function _computeWidth(chart, horizontal, width) {
      var widthInPx = width;

      try {
        if (typeof width === "string") {
          if (width.match(/px/)) {
            widthInPx = parseInt(width.replace(/px/, ""), 10);
          } else {
            // handle percentage values: convert to positive number between 0 and 100
            var widthInPercent = Math.min(
              100,
              Math.abs(Number(width.replace(/%/, ""))),
            );

            var model = chart.getDatasetMeta(0).data[0]._model;

            if (chart.config.type === "line") {
              widthInPx = parseInt(
                model.controlPointPreviousX + model.controlPointNextX,
                10,
              );
            } else if (horizontal) {
              widthInPx = parseInt(model.height, 10);
            } else if (!horizontal) {
              widthInPx = parseInt(model.width, 10);
            }

            widthInPx = (widthInPercent / 100) * widthInPx;
          }
        }
      } catch (e) {
        console.error(e);
      } finally {
        if (Number.isNaN(widthInPx)) {
          widthInPx = width;
        }
      }

      return widthInPx;
    },

    /**
     * draw error bar mark
     * @param ctx canvas context
     * @param model bar base coords
     * @param plus positive error bar position
     * @param minus negative error bar position
     * @param color error bar stroke color
     * @param width error bar width in pixel
     * @param lineWidth error ber line width
     * @param horizontal orientation
     * @private
     */
    _drawErrorBar: function _drawErrorBar(
      ctx,
      model,
      plus,
      minus,
      color,
      lineWidth,
      width,
      horizontal,
    ) {
      ctx.save();
      ctx.lineWidth = lineWidth;
      ctx.strokeStyle = color;
      ctx.lineWidth = lineWidth;
      ctx.beginPath();

      if (horizontal) {
        ctx.moveTo(minus, model.y - width / 2);
        ctx.lineTo(minus, model.y + width / 2);
        ctx.moveTo(minus, model.y);
        ctx.lineTo(plus, model.y);
        ctx.moveTo(plus, model.y - width / 2);
        ctx.lineTo(plus, model.y + width / 2);
      } else {
        ctx.moveTo(model.x - width / 2, plus);
        ctx.lineTo(model.x + width / 2, plus);
        ctx.moveTo(model.x, plus);
        ctx.lineTo(model.x, minus);
        ctx.moveTo(model.x - width / 2, minus);
        ctx.lineTo(model.x + width / 2, minus);
      }

      ctx.stroke();
      ctx.restore();
    },

    /**
     * plugin hook to draw the error bars
     * @param chart chartjs instance
     * @param easingValue animation function
     * @param options plugin options
     */
    afterDatasetsDraw: function afterDatasetsDraw(chart, easingValue, options) {
      var _this = this;

      // wait for easing value to reach 1 at the first render, after that draw immediately
      chart.__renderedOnce = chart.__renderedOnce || easingValue === 1;

      if (!chart.__renderedOnce) {
        return;
      }

      options = Object.assign({}, defaultOptions, options); // error bar and barchart bar coords

      var errorBarCoords = chart.data.datasets.map(function (d) {
        return d.errorBars;
      });

      var barchartCoords = this._getBarchartBaseCoords(chart);

      if (
        !barchartCoords ||
        !barchartCoords[0] ||
        !barchartCoords[0][0] ||
        !errorBarCoords
      ) {
        return;
      } // determine value scale and orientation (vertical or horizontal)

      var horizontal = this._isHorizontal(chart);

      var vScale = horizontal
        ? chart.scales["x-axis-0"]
        : chart.scales["y-axis-0"];
      var errorBarWidths = (
        Array.isArray(options.width) ? options.width : [options.width]
      ).map(function (w) {
        return _this._computeWidth(chart, horizontal, w);
      });
      var errorBarLineWidths = Array.isArray(options.lineWidth)
        ? options.lineWidth
        : [options.lineWidth];
      var errorBarColors = Array.isArray(options.color)
        ? options.color
        : [options.color];
      var ctx = chart.ctx;
      ctx.save(); // map error bar to barchart bar via label property

      barchartCoords.forEach(function (dataset, i) {
        if (
          chart.data.datasets[i]._meta != null &&
          chart.data.datasets[i]._meta.length > 0
        ) {
          var hidden = chart.data.dataset[i]._meta[0].hidden;

          if (hidden) {
            return;
          }
        }

        dataset.forEach(function (bar) {
          var cur = errorBarCoords[i];

          if (!cur) {
            return;
          }

          var hasLabelProperty = cur.hasOwnProperty(bar.label);
          var errorBarData = null; // common scale such as categorical

          if (hasLabelProperty) {
            errorBarData = cur[bar.label];
          } else if (
            !hasLabelProperty &&
            bar.label &&
            bar.label.label &&
            cur.hasOwnProperty(bar.label.label)
          ) {
            // hierarchical scale has its label property nested in b.label object as b.label.label
            errorBarData = cur[bar.label.label];
          }

          if (!errorBarData) {
            return;
          }

          var errorBars = Array.isArray(errorBarData)
            ? errorBarData
            : [errorBarData];
          var value = vScale.getRightValue(bar.value);
          errorBars.forEach(function (errorBar, ei) {
            // error bar data for the barchart bar or point in linechart
            var errorBarColor = errorBarColors[ei % errorBarColors.length]
              ? errorBarColors[ei % errorBarColors.length]
              : bar.color;
            var errorBarLineWidth =
              errorBarLineWidths[ei % errorBarLineWidths.length];
            var errorBarWidth = errorBarWidths[ei % errorBarWidths.length];
            var plusValue = options.absoluteValues
              ? errorBar.plus
              : value + Math.abs(errorBar.plus);
            var minusValue = options.absoluteValues
              ? errorBar.minus
              : value - Math.abs(errorBar.minus);
            var plus = vScale.getPixelForValue(plusValue);
            var minus = vScale.getPixelForValue(minusValue);

            _this._drawErrorBar(
              ctx,
              bar,
              plus,
              minus,
              errorBarColor,
              errorBarLineWidth,
              errorBarWidth,
              horizontal,
            );
          });
        });
      });
      ctx.restore();
    },
  };
  Chart.pluginService.register(ErrorBarsPlugin);

  return ErrorBarsPlugin;
});
