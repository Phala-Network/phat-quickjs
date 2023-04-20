import "./pink";
import "./polyfills";
import {
  parseTypes,
  createTupleEncoder,
  createEncoderForTypeId,
  createTupleDecoder,
  createDecoderForTypeId,
  encode,
  decode,
} from "./scale";

// TODO: lazy load the SCALE module
(function (g) {
  g.pink.SCALE = {
    parseTypes,
    createTupleEncoder,
    createEncoderForTypeId,
    createTupleDecoder,
    createDecoderForTypeId,
    encode,
    decode,
  };
})(globalThis as any);
