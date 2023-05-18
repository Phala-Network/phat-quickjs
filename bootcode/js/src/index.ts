import "./pink";
import "./polyfills";
import { parseTypes, codec } from "./scale";

// TODO: lazy load the SCALE module
(function (g) {
  g.pink.SCALE = { parseTypes, codec };
})(globalThis as any);
