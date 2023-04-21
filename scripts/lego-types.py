#!/usr/bin/env python3

import json

from itertools import chain
from collections import OrderedDict


class TypeInfo:
    id: str
    kind: str
    tdef: dict

    def __init__(self, info) -> None:
        self.id = info['id']
        td = info['type']['def']
        if 'primitive' in td:
            self.kind = 'primitive'
            self.tdef = td['primitive']
        elif 'composite' in td:
            self.kind = 'struct'
            self.tdef = td['composite']
        elif 'variant' in td:
            self.kind = 'enum'
            self.tdef = td['variant']
        elif 'array' in td:
            self.kind = 'array'
            self.tdef = td['array']
        elif 'tuple' in td:
            self.kind = 'tuple'
            self.tdef = td['tuple']
        elif 'sequence' in td:
            self.kind = 'vec'
            self.tdef = td['sequence']
        elif 'compact' in td:
            self.kind = 'compact'
            self.tdef = td['compact']
        else:
            raise Exception('Unknown type', td)

    def is_primitive(self):
        return self.kind == 'primitive'

    def __repr__(self) -> str:
        if self.is_primitive():
            return repr(self.tdef)
        return f'Type({self.id}, {self.kind}))'

    def to_type(self):
        if self.kind == 'primitive':
            return Primitive.load(self)
        elif self.kind == 'struct':
            return Struct.load(self)
        elif self.kind == 'enum':
            return Enum.load(self)
        elif self.kind == 'array':
            return Array.load(self)
        elif self.kind == 'tuple':
            return Tuple.load(self)
        elif self.kind == 'vec':
            return Vec.load(self)
        elif self.kind == 'compact':
            return Compact.load(self)
        else:
            raise Exception('Unknown type', self.kind)


class SymbolTable:
    def __init__(self, strip) -> None:
        self.next_index = 0
        self.symbols = OrderedDict()
        self.strip = strip

    def get(self, name):
        if name not in self.symbols:
            self.symbols[name] = self.next_index
            self.next_index += 1
        if self.strip:
            return self.symbols[name]
        else:
            return name


class Type:
    id = 0

    def is_primitive(self):
        return False


class Compact(Type):
    inner: Type

    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef['type'])

    def __init__(self, t) -> None:
        self.inner = t

    def __repr__(self) -> str:
        return f'Compat({self.inner})'

    def link(self, reg: 'Registry'):
        self.inner = reg.get_type(self.inner)

    def output(self, sym: 'SymbolTable'):
        return f'@{self.inner.id}'


class Vec(Type):
    inner: Type

    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef['type'])

    def __init__(self, t) -> None:
        self.inner = t

    def __repr__(self) -> str:
        return f'Vec({self.inner})'

    def link(self, reg: 'Registry'):
        self.inner = reg.get_type(self.inner)

    def output(self, sym: 'SymbolTable'):
        return f'[{self.inner.id}]'


class Array(Type):
    inner: Type
    len: int

    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef['type'], t.tdef['len'])

    def __init__(self, t, len) -> None:
        self.inner = t
        self.len = len

    def __repr__(self) -> str:
        return f'Array({self.inner}, {self.len})'

    def link(self, reg: 'Registry'):
        self.inner = reg.get_type(self.inner)

    def output(self, sym: 'SymbolTable'):
        return f'[{self.inner.id};{self.len}]'


class Tuple(Type):
    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef)

    def __init__(self, fields) -> None:
        self.fields = fields

    def __repr__(self) -> str:
        return f'Tuple({self.fields})'

    def link(self, reg: 'Registry'):
        self.fields = [reg.get_type(f) for f in self.fields]

    def output(self, sym: 'SymbolTable'):
        fields = ','.join(str(f.id) for f in self.fields)
        return f'({fields})'


class Primitive(Type):
    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef)

    def __init__(self, t) -> None:
        self.t = t

    def __repr__(self) -> str:
        return repr(self.t)

    def link(self, reg: 'Registry'):
        pass

    def is_primitive(self):
        return True

    def output(self, sym: 'SymbolTable'):
        return f'#{self.t}'


class Enum(Type):
    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef['variants'])

    def __init__(self, variants) -> None:
        self.variants = variants

    def __repr__(self) -> str:
        return f'Enum({self.variants})'

    def link(self, reg: 'Registry'):
        variants = []
        for v in self.variants:
            if 'fields' not in v:
                inner = None
            elif len(v['fields']) == 1 and 'name' not in v['fields'][0]:
                inner = reg.get_type(v['fields'][0]['type'])
            else:
                # Create a new struct type for the inner fields
                inner = Struct(v['fields'])
                inner.link(reg)
                reg.extra_types.append(inner)
            variants.append((v['index'], v['name'], inner))
        self.variants = variants

    def output(self, sym: 'SymbolTable'):
        variants = []
        for ord, (ind, name, t) in enumerate(self.variants):
            tref = t.id if t else ""
            symbol = sym.get(name)
            if ord == ind:
                variants.append(f'{symbol}:{tref}')
            else:
                variants.append(f'{symbol}:{tref}:{ind}')
        return f'<{",".join(variants)}>'


class Struct(Type):
    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef['fields'])

    def __init__(self, fields) -> None:
        self.fields = fields

    def __repr__(self) -> str:
        return f'Struct({self.fields})'

    def link(self, reg: 'Registry'):
        fields = []
        for f in self.fields:
            fields.append((f.get('name'), reg.get_type(f['type'])))
        self.fields = fields

    def output(self, sym: 'SymbolTable'):
        if self.fields and self.fields[0][0] is None:
            fields = ','.join(str(t.id) for (_, t) in self.fields)
            return f'({fields})'
        else:
            fields = ','.join(
                f'{sym.get(name)}:{t.id}' for (name, t) in self.fields)
            return f'{{{fields}}}'


class Registry:
    def __init__(self) -> None:
        self.types = {}
        self.extra_types = []

    def load(self, types):
        for t in types:
            typ = TypeInfo(t)
            self.types[typ.id] = typ

    def pre_link(self):
        for k in self.types:
            self.types[k] = self.types[k].to_type()

    def link(self):
        self.pre_link()
        for t in self.types.values():
            t.link(self)

    def export_types(self):
        for t in chain(self.types.values(), self.extra_types):
            # if t.is_primitive():
            #     continue
            yield t

    def reassign_ids(self):
        for id, t in enumerate(self.export_types()):
            t.id = id

    def compat(self, strip=False):
        self.reassign_ids()
        symbols = SymbolTable(strip)
        types = [t.output(symbols) for t in self.export_types()]
        symbols = list(symbols.symbols.keys())
        return types, symbols

    def get_type(self, id):
        return self.types[id]


def convert(types, strip=False):
    reg = Registry()
    reg.load(types)
    reg.link()
    compact_types, symbols = reg.compat(strip)
    output = "\n".join(compact_types)
    if strip:
        output += f'\nSymbols:\n{",".join(symbols)}'
    return output


if __name__ == '__main__':
    import sys
    metafile = sys.argv[1]
    contract = json.load(open(metafile))
    print(convert(contract['types'], 0))
