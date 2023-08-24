#!/usr/bin/env python
"""
A script that load js script and send it to the server.
Usage:
python load.py run [--url http://localhost:8003/push/message/0] <name> <script_name>
python load.py reset [--url http://localhost:8003/push/message/0] <name>

The payload is defined by the following Rust code.

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    Run { name: String, source: String },
    Reset { name: String },
}
"""

import argparse
import requests


def parse_args():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest='command')
    subparsers.required = True
    run = subparsers.add_parser('run')
    run.add_argument('name')
    run.add_argument('script_name')
    run.add_argument('--url', default='http://localhost:8003/push/message/0')
    reset = subparsers.add_parser('reset')
    reset.add_argument('name')
    reset.add_argument('--url', default='http://localhost:8003/push/message/0')
    return parser.parse_args()


def main():
    def run(args):
        with open(args.script_name, 'r') as f:
            source = f.read()
        payload = {'Run': {'name': args.name, 'source': source}}
        url = args.url
        print('Sending payload to {}'.format(url))
        print(payload)
        r = requests.post(url, json=payload)
        print('Response:')
        print(r.text)

    def reset(args):
        payload = {'Reset': {'name': args.name}}
        url = args.url
        print('Sending payload to {}'.format(url))
        print(payload)
        r = requests.post(url, json=payload)
        print('Response:')
        print(r.text)

    args = parse_args()
    if args.command == 'run':
        run(args)
    elif args.command == 'reset':
        reset(args)


if __name__ == '__main__':
    main()
