import "./bootcode";
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
