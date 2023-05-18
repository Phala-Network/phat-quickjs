import * as $ from "@scale-codec/core";

export interface Primitive {
  kind: "primitive";
  def: string;
}

export interface Compact {
  kind: "compact";
  def: number;
}

export interface Seq {
  kind: "seq";
  def: number;
}

export interface Tuple {
  kind: "tuple";
  def: number[];
}

export interface FixedArray {
  kind: "array";
  def: {
    type: number;
    size: number;
  };
}

export interface EnumVariant {
  index: number;
  name: string;
  type: number;
}

export interface Enum {
  kind: "enum";
  def: {
    variants: EnumVariant[];
  };
}

export interface StructField {
  name: string;
  type: number;
}

export interface Struct {
  kind: "struct";
  def: {
    fields: StructField[];
  };
}

export type ScaleType =
  | Primitive
  | Compact
  | Seq
  | Tuple
  | FixedArray
  | Enum
  | Struct;

function isU8(def: any): boolean {
  return def.kind === "primitive" && def.def === "u8";
}

function toU8(value: Uint8Array | string): Uint8Array {
  if (typeof value === "string") {
    if (value.startsWith("0x")) {
      return new Uint8Array(
        value
          .slice(2)
          .match(/.{1,2}/g)
          .map((byte) => parseInt(byte, 16))
      );
    } else {
      return new TextEncoder().encode(value);
    }
  }
  return value;
}

type TaggedValue = $.VariantAny;

function inlineToTagged(obj: any, meta: any): TaggedValue {
  let tag;
  let value;
  if (typeof obj === "string") {
    tag = obj; $.variant(obj);
    value = undefined;
  } else {
    tag = Object.keys(obj)[0];
    value = obj[tag];
  }
  const [_ind, type] = meta[tag];
  if (type == undefined) {
    return $.variant(tag);
  } else {
    return $.variant(tag, value);
  }
}

function taggedToInline(obj: TaggedValue): any {
  return { [obj.tag]: obj.content };
}

export function parseType(typeDef: string): ScaleType {
  if (typeDef.startsWith("#")) {
    const def = typeDef.slice(1);
    return { kind: "primitive", def };
  }

  if (typeDef.startsWith("@")) {
    const def = parseInt(typeDef.slice(1), 10);
    return { kind: "compact", def };
  }

  if (typeDef.startsWith("[") && typeDef.endsWith("]")) {
    if (typeDef.includes(";")) {
      const [id, size] = typeDef.slice(1, -1).split(";");
      return {
        kind: "array",
        def: { type: parseInt(id, 10), size: parseInt(size, 10) },
      };
    } else {
      const def = parseInt(typeDef.slice(1, -1), 10);
      return { kind: "seq", def };
    }
  }

  if (typeDef.startsWith("(") && typeDef.endsWith(")")) {
    const inner = typeDef.slice(1, -1);
    const def = inner ? inner.split(",").map((id) => parseInt(id, 10)) : [];
    return { kind: "tuple", def };
  }

  if (typeDef.startsWith("<") && typeDef.endsWith(">")) {
    const variants = typeDef
      .slice(1, -1)
      .split(",")
      .map((variant, ind) => {
        const [name, type, i] = variant.split(":");
        return {
          index: i === undefined ? ind : parseInt(i, 10),
          name,
          type: type ? parseInt(type, 10) : undefined,
        };
      });
    return { kind: "enum", def: { variants } };
  }

  if (typeDef.startsWith("{") && typeDef.endsWith("}")) {
    const fields = typeDef
      .slice(1, -1)
      .split(",")
      .map((field) => {
        const [name, type] = field.split(":");
        return { name, type: parseInt(type, 10) };
      });
    return { kind: "struct", def: { fields } };
  }

  throw new Error("Invalid type definition");
}

export function parseTypes(typeDefs: string): ScaleType[] {
  const typeDefLines = typeDefs
    .split("\n")
    .filter((line) => line.trim() !== "");
  return typeDefLines.map(parseType);
}

export interface ScaleTypeRegistry {
  [index: number]: ScaleType;
}

export function createEncoderForTypeId(
  typeId: number,
  registry: ScaleTypeRegistry
): any {
  const typeDef = registry[typeId];
  if (typeDef === undefined) {
    throw new Error(`Unknown type id: ${typeId}`);
  }
  return createEncoder(typeDef, registry);
}

function createEncoder(typeDef: ScaleType, registry: ScaleTypeRegistry): any {
  switch (typeDef.kind) {
    case "primitive":
      return createPrimitiveEncoder(typeDef.def);
    case "compact":
      return createCompactEncoder();
    case "seq":
      return createSeqEncoder(typeDef.def, registry);
    case "tuple":
      return createTupleEncoder(typeDef.def, registry);
    case "array":
      return createArrayEncoder(typeDef.def.type, typeDef.def.size, registry);
    case "enum":
      return createEnumEncoder(typeDef.def.variants, registry);
    case "struct": {
      return createStructEncoder(typeDef.def.fields, registry);
    }
    default:
      throw new Error("Invalid type definition");
  }
}

export function createDecoderForTypeId(
  typeId: number,
  registry: ScaleTypeRegistry
): any {
  const typeDef = registry[typeId];
  if (typeDef === undefined) {
    throw new Error(`Unknown type id: ${typeId}`);
  }
  return createDecoder(typeDef, registry);
}

function createDecoder(typeDef: ScaleType, registry: ScaleTypeRegistry): any {
  switch (typeDef.kind) {
    case "primitive":
      return createPrimitiveDecoder(typeDef.def);
    case "compact":
      return createCompactDecoder();
    case "seq":
      return createSeqDecoder(typeDef.def, registry);
    case "tuple":
      return createTupleDecoder(typeDef.def, registry);
    case "array":
      return createArrayDecoder(typeDef.def.type, typeDef.def.size, registry);
    case "enum":
      return createEnumDecoder(typeDef.def.variants, registry);
    case "struct":
      return createStructDecoder(typeDef.def.fields, registry);
    default:
      throw new Error("Invalid type definition");
  }
}

const INT_BYTES_COUNT_MAP = {
  i8: 1,
  u8: 1,
  i16: 2,
  u16: 2,
  i32: 4,
  u32: 4,
  i64: 8,
  u64: 8,
  i128: 16,
  u128: 16,
  i256: 32,
  u256: 32,
  i512: 64,
  u512: 64,
};

const tySizeHint = (ty: $.BigIntTypes) => () => INT_BYTES_COUNT_MAP[ty];

function bienc(ty: $.BigIntTypes): $.Encode<bigint> {
  return $.encodeFactory(
    (value: number | bigint, writer) =>
      $.encodeBigInt(BigInt(value), ty, writer),
    tySizeHint(ty)
  );
}

function createPrimitiveEncoder(def: string): any {
  return {
    bool: $.encodeBool,
    u8: $.encodeU8,
    u16: $.encodeU16,
    u32: $.encodeU32,
    u64: bienc("u64"),
    u128: bienc("u128"),
    u256: bienc("u256"),
    i8: $.encodeI8,
    i16: $.encodeI16,
    i32: $.encodeI32,
    i64: bienc("i64"),
    i128: bienc("i128"),
    i256: bienc("i256"),
    str: $.encodeStr,
  }[def];
}

function createPrimitiveDecoder(def: string): any {
  return {
    bool: $.decodeBool,
    u8: $.decodeU8,
    u16: $.decodeU16,
    u32: $.decodeU32,
    u64: $.decodeU64,
    u128: $.decodeU128,
    i8: $.decodeI8,
    i16: $.decodeI16,
    i32: $.decodeI32,
    i64: $.decodeI64,
    i128: $.decodeI128,
    str: $.decodeStr,
  }[def];
}

function createCompactEncoder(): any {
  return $.encodeCompact;
}

function createCompactDecoder(): any {
  return $.decodeCompact;
}

export function createTupleEncoder(
  def: number[],
  registry: ScaleTypeRegistry
): any {
  const encoder = $.createTupleEncoder(
    def.map((typeId) => createEncoder(registry[typeId], registry)) as any
  );
  return $.encodeFactory(
    (value: any, walker: any) => {
      return encoder(value || [], walker);
    },
    (value: any) => {
      return encoder.sizeHint(value || []);
    }
  );
}

export function createTupleDecoder(
  def: number[],
  registry: ScaleTypeRegistry
): any {
  return $.createTupleDecoder(
    def.map((typeId) => createDecoder(registry[typeId], registry)) as any
  );
}

export function createArrayEncoder(
  inner: number,
  size: number,
  registry: ScaleTypeRegistry
): any {
  const def = registry[inner];
  const isU8Array = isU8(def);
  const arrayEncoder = $.createArrayEncoder(createEncoder(def, registry), size);
  return $.encodeFactory(
    (value: any, walker: any) => {
      if (isU8Array) {
        value = toU8(value);
      }
      return arrayEncoder(value, walker);
    },
    (value: any) => {
      if (isU8Array) {
        value = toU8(value);
      }
      return arrayEncoder.sizeHint(value);
    }
  );
}

export function createArrayDecoder(
  inner: number,
  size: number,
  registry: ScaleTypeRegistry
): any {
  return $.createArrayDecoder(createDecoder(registry[inner], registry), size);
}

export function createSeqEncoder(
  inner: number,
  registry: ScaleTypeRegistry
): any {
  const def = registry[inner];
  const isU8Vector = isU8(def);
  const vecEncoder = $.createVecEncoder(createEncoder(def, registry));
  return $.encodeFactory(
    (value: any, walker: any) => {
      if (isU8Vector) {
        value = toU8(value);
      }
      return vecEncoder(value, walker);
    },
    (value: any) => {
      if (isU8Vector) {
        value = toU8(value);
      }
      return vecEncoder.sizeHint(value);
    }
  );
}

export function createSeqDecoder(
  inner: number,
  registry: ScaleTypeRegistry
): any {
  const def = registry[inner];
  const isU8Vector = isU8(def);
  const vecDecoder = $.createVecDecoder(createDecoder(def, registry));
  return (reader: any) => {
    const value = vecDecoder(reader);
    return isU8Vector ? new Uint8Array(value as any) : value;
  };
}

function createEnumEncoder(
  variants: EnumVariant[],
  registry: ScaleTypeRegistry
): any {
  const meta: any = {};
  for (const variant of variants) {
    meta[variant.name] = [
      variant.index,
      variant.type !== undefined
        ? createEncoder(registry[variant.type], registry)
        : undefined,
    ];
  }
  const encoder = $.createEnumEncoder(meta);
  return $.encodeFactory(
    (value: any, walker: any) =>
      encoder(inlineToTagged(value, meta) as any, walker),
    (value: any) => encoder.sizeHint(inlineToTagged(value, meta) as any)
  );
}

function createEnumDecoder(
  variants: EnumVariant[],
  registry: ScaleTypeRegistry
): any {
  const meta: any = {};
  for (const variant of variants) {
    meta[variant.index] = [
      variant.name,
      variant.type !== undefined
        ? createDecoder(registry[variant.type], registry)
        : undefined,
    ];
  }
  const decoder = $.createEnumDecoder(meta);
  return (reader: $.Walker) => {
    return taggedToInline(decoder(reader));
  };
}

function createStructEncoder(
  fields: StructField[],
  registry: ScaleTypeRegistry
): any {
  const encoders = fields.map((field) => [
    field.name,
    createEncoder(registry[field.type], registry),
  ]);
  return $.createStructEncoder(encoders as any);
}

function createStructDecoder(
  fields: StructField[],
  registry: ScaleTypeRegistry
): any {
  const decoders = fields.map((field) => [
    field.name,
    createDecoder(registry[field.type], registry),
  ]);
  return $.createStructDecoder(decoders as any);
}

interface Codec {
  encode(value: any): Uint8Array;
  decode(source: Uint8Array): any;
}

export function codec(
  typeId: number | number[],
  registry: ScaleTypeRegistry
): Codec {
  return {
    encode: (value: any) => {
      let encoder;
      if (Array.isArray(typeId)) {
        encoder = createTupleEncoder(typeId, registry);
      } else {
        encoder = createEncoderForTypeId(typeId, registry);
      }
      return encode(value, encoder);
    },
    decode: (source: Uint8Array) => {
      let decoder;
      if (Array.isArray(typeId)) {
        decoder = createTupleDecoder(typeId, registry);
      } else {
        decoder = createDecoderForTypeId(typeId, registry);
      }
      return decode(source, decoder);
    },
  };
}

function encode(value: any, encode: any): Uint8Array {
  return $.WalkerImpl.encode(value, encode);
}

function decode(source: any, decode: any): any {
  const walker = new $.WalkerImpl(source);
  return decode(walker);
}
