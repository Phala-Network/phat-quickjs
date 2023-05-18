import * as scale from "scale-codec";
import { ScaleAssertError } from "scale-codec";

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

export function toBytes(value: Uint8Array | string): Uint8Array {
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

interface TaggedValue {
  tag: string;
  value?: any;
}

function inlineToTagged(obj: any): TaggedValue {
  let tag, value, isEmpty;
  if (obj.tag) {
    tag = obj.tag;
    value = obj.value;
  } else {
    tag = Object.keys(obj)[0];
    value = obj[tag];
  }
  return { tag, value };
}

function taggedToInline(taggedObj: TaggedValue): any {
  const { tag, value } = taggedObj;
  return { [tag]: value };
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

interface ScaleTypeRegistry {
  [index: number]: ScaleType;
}

export function codec(
  typeId: number | number[],
  registry: ScaleTypeRegistry
): any {
  if (Array.isArray(typeId)) {
    return tupleCodec(typeId, registry);
  }
  const typeDef = registry[typeId];
  if (typeDef === undefined) {
    throw new Error(`Unknown type id: ${typeId}`);
  }
  return anyCodec(typeDef, registry);
}

function anyCodec(def: ScaleType, registry: ScaleTypeRegistry): any {
  switch (def.kind) {
    case "primitive":
      return primitiveCodec(def.def);
    case "compact":
      return compactCodec(def.def, registry);
    case "seq":
      return seqCodec(def.def, registry);
    case "tuple":
      return tupleCodec(def.def, registry);
    case "array":
      return arrayCodec(def.def.type, def.def.size, registry);
    case "enum":
      return enumCodec(def.def.variants, registry);
    case "struct":
      return structCodec(def.def.fields, registry);
    default:
      throw new Error("Invalid type definition");
  }
}

function primitiveCodec(def: string): any {
  const {
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    u256,
    i8,
    i16,
    i32,
    i64,
    i128,
    i256,
    str,
  } = scale;
  return {
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    u256,
    i8,
    i16,
    i32,
    i64,
    i128,
    i256,
    str,
  }[def];
}

function compactCodec(innerId: number, registry: ScaleTypeRegistry): any {
  return scale.compact(anyCodec(registry[innerId], registry));
}

function tupleCodec(def: number[], registry: ScaleTypeRegistry): any {
  return scale.tuple(
    ...def.map((typeId) => anyCodec(registry[typeId], registry)) as any
  );
}

function arrayCodec(
  inner: number,
  size: number,
  registry: ScaleTypeRegistry
): any {
  const def = registry[inner];
  if (isU8(def)) {
    return bytesCodec(scale.sizedUint8Array(size));
  } else {
    return scale.sizedArray(anyCodec(registry[inner], registry), size);
  }
}

function seqCodec(inner: number, registry: ScaleTypeRegistry): any {
  const def = registry[inner];
  if (isU8(def)) {
    return bytesCodec(scale.uint8Array);
  } else {
    return scale.array(anyCodec(def, registry));
  }
}

function enumCodec(variants: EnumVariant[], registry: ScaleTypeRegistry): any {
  const inner = scale.taggedUnion(
    "tag",
    variants.map((variant) => {
      return variant.type !== undefined
        ? scale.variant(
            variant.name,
            scale.field("value", anyCodec(registry[variant.type], registry))
          )
        : scale.variant(variant.name);
    })
  );
  return scale.createCodec<Uint8Array | string>({
    _metadata: scale.metadata("enum"),
    _staticSize: inner._staticSize,
    _decode(buffer) {
      return taggedToInline(inner._decode(buffer));
    },
    _encode(buffer, value) {
      return inner._encode(buffer, inlineToTagged(value));
    },
    _assert(assert) {
      assert.value = inlineToTagged(assert.value);
      inner._assert(assert);
    },
  });
}

function structCodec(fields: StructField[], registry: ScaleTypeRegistry): any {
  const _$ = scale;
  return _$.object(
    ...fields.map((field) =>
      _$.field(field.name, anyCodec(registry[field.type], registry))
    )
  );
}

function bytesCodec(
  inner: scale.Codec<Uint8Array>
): scale.Codec<Uint8Array | string> {
  return scale.createCodec<Uint8Array | string>({
    _metadata: scale.metadata("bytes"),
    _staticSize: inner._staticSize,
    _decode: inner._decode,
    _encode(buffer, value) {
      return inner._encode(buffer, toBytes(value));
    },
    _assert(assert) {
      if (
        !(
          assert.value instanceof Uint8Array || typeof assert.value === "string"
        )
      ) {
        throw new ScaleAssertError(
          this,
          assert.value,
          `${assert.path} is not a Uint8Array or string`
        );
      }
    },
  });
}
