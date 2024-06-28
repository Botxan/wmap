#!/usr/bin/env python3
import sys
import os
import subprocess
import argparse
import re

def get_host_port():
    result = subprocess.getoutput("docker-compose port web 80")
    match = re.search(r':(\d+)$', result)
    if match:
        return match.group(1)
    else:
        print("Could not determine host port.")
        sys.exit(1)

host_port = get_host_port()
base_url = f"http://127.0.0.1:{host_port}/wmap"

frameworks_list = {
    "cakephp-4.5": base_url + "/cakephp-4.5/webroot/index.php/hello/index",
    "cakephp-5.0": base_url + "/cakephp-5.0/webroot/index.php/hello/index",
    "codeigniter-4.4": base_url + "/codeigniter-4.4/public/index.php/hello/index",
    "fastroute-1.3": base_url + "/fastroute-1.3/public/index.php/hello/index",
    "fatfree-3.8": base_url + "/fatfree-3.8/public/index.php/hello/index",
    "fuelphp-1.9": base_url + "/fuelphp-1.9/public/index.php/helloworld/index",
    "kumbia-1.1": base_url + "/kumbia-1.1/default/public/index.php/helloworld/index",
    "laravel-10.2": base_url + "/laravel-10.2/public/index.php/hello/index",
    "leaf-3.5": base_url + "/leaf-3.5/public/index.php/hello/index",
    "lumen-10.0": base_url + "/lumen-10.0/public/index.php/hello/index",
    "phroute-2.2": base_url + "/phroute-2.2/public/index.php/hello/index",
    "pure-php": base_url + "/pure-php/public/index.php/hello/index",
    "silex-2.3": base_url + "/silex-2.3/web/index.php/hello/index",
    "slim-4.12": base_url + "/slim-4.12/public/index.php/hello/index",
    "symfony-5.4": base_url + "/symfony-5.4/public/index.php/hello/index",
    "symfony-6.4": base_url + "/symfony-6.4/public/index.php/hello/index",
    "symfony-7.0": base_url + "/symfony-7.0/public/index.php/hello/index",
    "ubiquity-2.4.x.dev": base_url + "/ubiquity-2.4.x.dev/public/index.php?c=HelloWorldController/index",
    "yii-2.0-basic": base_url + "/yii-2.0-basic/web/index.php?r=helloworld/index"
}

GREEN = '\033[0;32m'
RED = '\033[0;31m'
NC = '\033[0m' # No Color

def trim_text(text, trim_direction=None):
    if trim_direction == 'r':
        return text.rstrip()
    elif trim_direction == 'l':
        return text.lstrip()
    else:
        return text.strip()

def str_replace(search, replace, text):
    return text.replace(search, replace)

def show_help():
    help_text = '''
Usage: python3 check.py [-t pure-php slim-*]

Optional Arguments:
    -h, --help                  Show this help message and exit
    -t, --target                Specify your target framework/s
                                Separate them by spaces.
'''
    print(help_text)


def main():

    parser = argparse.ArgumentParser(description="Frameworks", add_help=False)
    parser.add_argument("-t", "--target", nargs='*', help="Specify your target framework/s")
    parser.add_argument("-h", "--help", action='store_true', help="Show this help message and exit")

    args = parser.parse_args()

    if args.help:
        show_help()
        sys.exit(1)

    param_targets = frameworks_list.keys()

    if args.target:
        param_targets = args.target

    fail = 0

    for fw in param_targets:
        url = frameworks_list.get(fw)
        if url:
            url_output = subprocess.getoutput(f"curl -s {url}")
            # Expected to get the Hello Wmap!
            if not re.match(r'^Hello Wmap!$', url_output):
                print(f"{RED}❌ {fw} {NC}")
                print(f"{url}")

                if subprocess.call(["which", "w3m"], stdout=subprocess.PIPE, stderr=subprocess.PIPE) == 0:
                    os.system(f"echo \"{url_output}\" | w3m -dump -T text/html")
                else:
                    print(url_output)

                fail = 1
            else:
                print(f"{GREEN}✔ {fw}{NC} {len(url_output)} bytes   {url}")
        else:
            print(f"{RED}Could not find URL for {fw}{NC}")

    sys.exit(fail)

if __name__ == "__main__":
    main()
