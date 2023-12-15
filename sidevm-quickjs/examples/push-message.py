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
    parser.add_argument('--url', default='http://localhost:8003')
    subparsers = parser.add_subparsers(dest='command')
    subparsers.required = True
    run = subparsers.add_parser('run')
    run.add_argument('--id', default=0)
    run.add_argument('name')
    run.add_argument('script_name')
    reset = subparsers.add_parser('reset')
    reset.add_argument('--id', default=0)
    reset.add_argument('name')
    deploy = subparsers.add_parser('deploy')
    deploy.add_argument('--id', default=None)
    deploy.add_argument('wasm_file')
    stop = subparsers.add_parser('stop')
    stop.add_argument('id')
    return parser.parse_args()


def main():
    def run(args):
        with open(args.script_name, 'r') as f:
            source = f.read()
        payload = {'Run': {'name': args.name, 'source': source}}
        url = f"{args.url}/push/message/{args.id}"
        print('Sending payload to {}'.format(url))
        print(payload)
        r = requests.post(url, json=payload)
        print('Response:')
        print(r.text)

    def reset(args):
        payload = {'Reset': {'name': args.name}}
        url = args.url + '/push/message/' + args.id
        print('Sending payload to {}'.format(url))
        print(payload)
        r = requests.post(url, json=payload)
        print('Response:')
        print(r.text)

    def deploy(args):
        url = args.url
        if args.id:
            url += '/run?id=' + args.id
        print('Sending payload to {}'.format(url))
        with open(args.wasm_file, 'rb') as f:
            wasm = f.read()
            r = requests.post(url, data=wasm)
            print('Response:')
            print(r.text)

    def stop(args):
        url = args.url
        if not args.id:
            raise Exception("Missing VM id")
        url += '/stop?id=' + args.id
        print('Sending payload to {}'.format(url))
        r = requests.post(url, data='')
        print('Response:')
        print(r.text)

    args = parse_args()
    if args.command == 'run':
        run(args)
    elif args.command == 'reset':
        reset(args)
    elif args.command == 'deploy':
        deploy(args)
    elif args.command == 'stop':
        stop(args)


if __name__ == '__main__':
    main()
