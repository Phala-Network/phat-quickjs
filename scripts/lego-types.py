#!/usr/bin/env python3

import sys
import json
import argparse

from itertools import chain
from collections import OrderedDict
from typing import List


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
    _id = 0
    alias: 'Type' = None

    def is_primitive(self):
        return False

    def __eq__(self, __value: object) -> bool:
        if self is __value:
            return True
        if not isinstance(__value, Type):
            return False
        if self.alias is not None:
            return self.alias == __value
        if __value.alias is not None:
            return self == __value.alias
        return self.eq(__value)

    @property
    def id(self):
        if self.alias is not None:
            return self.alias.id
        return self._id

    @id.setter
    def id(self, value):
        self._id = value

    def real_type(self):
        if self.alias is not None:
            return self.alias.real_type()
        return self

    def pick_types(self):
        yield self

    def __hash__(self) -> int:
        return hash(self.id)


class Compact(Type):
    inner: Type

    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef['type'])

    def __init__(self, t) -> None:
        self.inner = t

    def __repr__(self) -> str:
        return f'Compat({self.inner})'

    def link(self, reg: 'Contract'):
        self.inner = reg.get_type(self.inner)

    def output(self, sym: 'SymbolTable'):
        return f'@{self.inner.id}'

    def eq(self, value: object) -> bool:
        return isinstance(value, Compact) and value.inner == self.inner

    def pick_types(self):
        yield from self.inner.pick_types()
        yield self


class Vec(Type):
    inner: Type

    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef['type'])

    def __init__(self, t) -> None:
        self.inner = t

    def __repr__(self) -> str:
        return f'Vec({self.inner})'

    def link(self, reg: 'Contract'):
        self.inner = reg.get_type(self.inner)

    def output(self, sym: 'SymbolTable'):
        return f'[{self.inner.id}]'

    def eq(self, value: object) -> bool:
        return isinstance(value, Vec) and value.inner == self.inner

    def pick_types(self):
        yield from self.inner.pick_types()
        yield self


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

    def link(self, reg: 'Contract'):
        self.inner = reg.get_type(self.inner)

    def output(self, sym: 'SymbolTable'):
        return f'[{self.inner.id};{self.len}]'

    def eq(self, value: object) -> bool:
        return isinstance(value, Array) and value.inner == self.inner and value.len == self.len

    def pick_types(self):
        yield from self.inner.pick_types()
        yield self


class Tuple(Type):
    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef)

    def __init__(self, fields) -> None:
        self.fields = fields

    def __repr__(self) -> str:
        return f'Tuple({self.fields})'

    def link(self, reg: 'Contract'):
        self.fields = [reg.get_type(f) for f in self.fields]

    def output(self, sym: 'SymbolTable'):
        fields = ','.join(str(f.id) for f in self.fields)
        return f'({fields})'

    def eq(self, value: object) -> bool:
        return isinstance(value, Tuple) and value.fields == self.fields

    def pick_types(self):
        for f in self.fields:
            yield from f.pick_types()
        yield self


class Primitive(Type):
    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef)

    def __init__(self, t) -> None:
        self.t = t

    def __repr__(self) -> str:
        return repr(self.t)

    def link(self, reg: 'Contract'):
        pass

    def is_primitive(self):
        return True

    def output(self, sym: 'SymbolTable'):
        return f'#{self.t}'

    def eq(self, value: object) -> bool:
        return isinstance(value, Primitive) and value.t == self.t

    def pick_types(self):
        yield self


class Enum(Type):
    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef['variants'])

    def __init__(self, variants) -> None:
        self.variants = variants

    def __repr__(self) -> str:
        return f'Enum({self.variants})'

    def link(self, reg: 'Contract'):
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

    def eq(self, value: object) -> bool:
        return isinstance(value, Enum) and value.variants == self.variants

    def pick_types(self):
        for _, _, t in self.variants:
            if t:
                yield from t.pick_types()
        yield self


class Struct(Type):
    @classmethod
    def load(cls, t: 'TypeInfo'):
        return cls(t.tdef['fields'])

    def __init__(self, fields) -> None:
        self.fields = fields

    def __repr__(self) -> str:
        return f'Struct({self.fields})'

    def link(self, reg: 'Contract'):
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

    def eq(self, value: object) -> bool:
        return isinstance(value, Struct) and value.fields == self.fields

    def pick_types(self):
        for _, t in self.fields:
            yield from t.pick_types()
        yield self


class Message:
    contract: 'Contract'
    name: str
    selector: str
    inputs: List[Type]
    output: Type

    @classmethod
    def from_metadata(cls, contract: 'Contract', msg: dict):
        inputs = [arg['type']['type'] for arg in msg['args']]
        output = msg['returnType']['type']
        return Message(contract, msg['label'], msg['selector'], inputs,
                       output)

    def __init__(self, contract, name, selector, inputs, output) -> None:
        self.contract = contract
        self.name = name
        self.selector = selector
        self.inputs = inputs
        self.output = output

    def link(self, reg: 'Contract'):
        self.inputs = [reg.get_type(t) for t in self.inputs]
        self.output = reg.get_type(self.output)

    def pick_types(self):
        for t in self.inputs:
            yield from t.pick_types()
        yield from self.output.pick_types()

    def display(self) -> str:
        inputs = ', '.join([self.selector] + [str(t.id)
                                              for t in self.inputs])
        return f'{self.contract}.{self.name}({inputs}) -> {self.output.id}'


class Contract:
    def __init__(self, metadata) -> None:
        self.metadata = metadata
        self.name = metadata['contract']['name']
        self.types = {}
        self.extra_types = []
        self.messages = []  # type: List[Message]
        self.constructors = []  # type: List[Message]
        self.load(metadata)

    def load(self, metadata):
        for t in metadata['types']:
            typ = TypeInfo(t)
            self.types[typ.id] = typ

        spec = metadata['spec']
        for c in spec['constructors']:
            self.constructors.append(Message.from_metadata(self, c))

        for m in spec['messages']:
            self.messages.append(Message.from_metadata(self, m))

    def pre_link(self):
        for k in self.types:
            self.types[k] = self.types[k].to_type()

    def link(self):
        self.pre_link()
        for t in self.types.values():
            t.link(self)
        for m in chain(self.constructors, self.messages):
            m.link(self)

    def export_types(self):
        for t in chain(self.types.values(), self.extra_types):
            # if t.is_primitive():
            #     continue
            yield t

    def get_type(self, id):
        return self.types[id]

    def __str__(self) -> str:
        return f'Contract({self.name})'


class Registry:
    def __init__(self, contracts: 'List[Contract]') -> None:
        self.contracts = contracts

    def link(self):
        for contract in self.contracts:
            contract.link()

    def export_types(self):
        for contract in self.contracts:
            for t in contract.export_types():
                if t.alias is not None:
                    continue
                yield t

    def merge_types(self):
        for i in range(100):
            types = []
            alised = 0
            for t in self.export_types():
                if t.alias is not None:
                    continue
                for ref in types:
                    if t.eq(ref):
                        print(f'Aliased {t.id} to {ref.id}')
                        t.alias = ref
                        alised += 1
                        break
                else:
                    types.append(t)
            print(f'Iteration {i}: {alised} aliased types')
            if alised == 0:
                break

    def all_messages(self):
        for contract in self.contracts:
            for m in chain(contract.constructors, contract.messages):
                yield m

    def find_messages(self, paths: List[str]):
        for message in paths:
            for c in self.contracts:
                for m in c.messages:
                    if m.selector == message if message.startswith('0x') else m.name == message:
                        yield m
                        break


def reassign_ids(types):
    for id, t in enumerate(types):
        t.id = id


def compat(types, strip=False):
    types = list(types)
    reassign_ids(types)
    symbols = SymbolTable(strip)
    types = [t.output(symbols) for t in types]
    symbols = list(symbols.symbols.keys())
    return types, symbols


def compat_and_print(types):
    types = list(types)
    types, _symbols = compat(types)
    print("Types:")
    print("\n".join([f'{i}: {t}' for i, t in enumerate(types)]))
    print("----")
    print("\n".join(types))
    print("----")


def print_messages(messages):
    print("Messages:")
    for m in messages:
        print(m.display())


def parse_args():
    parser = argparse.ArgumentParser(
        description='A CLI tool for processing lego types for contracts.')

    # Function to split messages by comma
    def parse_messages(messages_str):
        return messages_str.split(',')
    parser.add_argument(
        '-k', '--keep',
        dest='keep_messages',
        metavar='MESSAGES',
        type=parse_messages,
        help='Specify a comma-separated list of messages to keep (e.g., message1,message2,...)'
    )
    parser.add_argument(
        'contract_files',
        metavar='CONTRACT_FILES',
        nargs='+',
        help='One or more contract files to be processed'
    )

    return parser.parse_args()


def main():
    args = parse_args()
    contracts = [Contract(json.load(open(f))) for f in args.contract_files]
    registry = Registry(contracts)
    registry.link()

    compat_and_print(registry.export_types())
    print_messages(registry.all_messages())
    print("Reduced:")
    registry.merge_types()
    compat_and_print(registry.export_types())
    print_messages(registry.all_messages())

    if args.keep_messages:
        print("Eliminated:")
        messages = list(registry.find_messages(args.keep_messages))
        all_types = set()
        for m in messages:
            all_types.update(m.pick_types())
        compat_and_print(all_types)
        print_messages(messages)


if __name__ == '__main__':
    main()
