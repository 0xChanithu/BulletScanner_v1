#!/usr/bin/env python3
import argparse,ipaddress,os,shutil,subprocess,sys
def rng(v):
 try:
  a,b=map(int,v.split('-',1)); assert 1<=a<=b<=65535; return a,b
 except: raise argparse.ArgumentTypeError('Use a range such as 1-10000')
p=argparse.ArgumentParser(description='MiniScanner X v2'); p.add_argument('target'); p.add_argument('--ports',default='1-10000',type=rng); p.add_argument('--concurrency',type=int,default=500); p.add_argument('--timeout-ms',type=int,default=700); p.add_argument('--retries',type=int,default=1); p.add_argument('--no-os',action='store_true'); a=p.parse_args()
try: ipaddress.ip_address(a.target)
except: print('[!] Invalid IP address'); sys.exit(2)
b=os.path.expanduser('~/.local/bin/miniscanner-portscan'); b=b if os.path.isfile(b) else shutil.which('miniscanner-portscan')
if not b: print('[!] Run ./install.sh first'); sys.exit(1)
start,end=a.ports; r=subprocess.run([b,a.target,'--start',str(start),'--end',str(end),'--concurrency',str(max(1,a.concurrency)),'--timeout-ms',str(max(100,a.timeout_ms)),'--retries',str(max(0,a.retries))],capture_output=True,text=True)
if r.returncode: print(r.stderr); sys.exit(r.returncode)
ports=sorted({x.strip() for x in r.stdout.splitlines() if x.strip().isdigit()},key=int)
if not ports: print('[+] No open TCP ports found'); sys.exit(0)
print('[+] Open ports:',','.join(ports)); cmd=['nmap','-sV','--open','-p',','.join(ports)];
if not a.no_os: cmd.insert(1,'-O')
cmd.append(a.target); sys.exit(subprocess.run(cmd).returncode)
