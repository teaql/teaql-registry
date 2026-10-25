#!/usr/bin/env python3
"""Native cold/cache/offline verification with a disconnectable upstream gateway.
Only disposable localhost PostgreSQL and file-blob state are used. All client
caches are fresh per phase; the registry is restarted before offline verification.
"""
import argparse, base64, ssl, gzip, hashlib, http.server, json, os, pathlib, re, shutil, socket, subprocess, threading, time, urllib.error, urllib.parse, urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
URL = 'http://127.0.0.1:28441'
GATEWAY = 'http://127.0.0.1:28445'
ALL = ['cargo','pypi','gomod','nuget','docker','swift','dart','rubygems','composer','conan','hex','raw']
TARGETS = {'cargo':'https://index.crates.io/','pypi':'https://pypi.org/','gomod':'https://proxy.golang.org/', 'sumdb':'https://sum.golang.org/', 'nuget':'https://api.nuget.org/', 'docker':'https://registry-1.docker.io/', 'dart':'https://pub.dev/', 'rubygems':'https://rubygems.org/', 'composer':'https://repo.packagist.org/', 'hex':'https://repo.hex.pm/', 'conan':URL+'/repository/fixture-conan/conan/', 'swift':URL+'/repository/fixture-swift/swift/', 'raw':URL+'/repository/fixture-raw/', 'npm':'https://registry.npmjs.org/', 'maven2':'https://repo.maven.apache.org/maven2/'}

class Gateway(http.server.BaseHTTPRequestHandler):
    mappings = {}
    counts = {}
    guard = threading.Lock()
    logpath = None
    def log_message(self, *_): pass
    @classmethod
    def bind(cls, protocol, url):
        # Preserve template suffixes while recording their directory base.
        fragment = urllib.parse.urlsplit(url).fragment
        url = urllib.parse.urldefrag(url)[0]
        marker = min([url.find(x) for x in ['%package%','%hash%'] if x in url] or [len(url)])
        template = ''
        if marker < len(url):
            split = url.rfind('/',0,marker)+1
            url, template = url[:split], url[split:]
        directory = urllib.parse.urlsplit(url).path.endswith('/')
        key = hashlib.sha256((protocol+':'+url).encode()).hexdigest()
        filename = urllib.parse.urlsplit(url).path.rsplit('/',1)[-1] or 'download'
        with cls.guard: cls.mappings[key]=(protocol,url,directory,filename)
        tail = template if template else ('' if directory else filename)
        return f'{GATEWAY}/linked/{key}/{tail}'+('#'+fragment if fragment else '')
    @classmethod
    def rewrite(cls, protocol, value, source):
        if isinstance(value,dict):
            return {k:cls.rewrite(protocol,v,source) for k,v in value.items()}
        if isinstance(value,list): return [cls.rewrite(protocol,v,source) for v in value]
        if isinstance(value,str) and value.startswith(('https://','http://')) and not value.startswith(GATEWAY):
            # Signed Hex data and npm documents do not use this transform.
            return cls.bind(protocol,value)
        return value
    def do_GET(self):
        parsed=urllib.parse.urlsplit(self.path)
        fields=parsed.path.lstrip('/').split('/',2)
        if fields[0]=='linked':
            mapping=self.mappings.get(fields[1])
            if not mapping: self.send_error(404); return
            protocol,base,directory,filename=mapping
            tail=fields[2] if len(fields)>2 else ''
            target=urllib.parse.urljoin(base,tail) if directory else base
            if not directory and tail==filename+'.metadata': target += '.metadata'
            elif not directory and tail.startswith(filename+'/'): target=base.rstrip('/')+'/'+tail[len(filename)+1:]
        else:
            protocol=fields[0]
            if protocol not in TARGETS: self.send_error(404); return
            tail=parsed.path[len(protocol)+2:]
            target=urllib.parse.urljoin(TARGETS[protocol],tail)
        if protocol=='sumdb': protocol='gomod'
        if parsed.query: target += ('&' if '?' in target else '?')+parsed.query
        headers={'Accept-Encoding':'identity','User-Agent':'TeaQL-Proxy-Native-Verification/1'}
        for name in ['Accept','Authorization']:
            if self.headers.get(name): headers[name]=self.headers[name]
        try:
            response=urllib.request.urlopen(urllib.request.Request(target,headers=headers),timeout=90)
        except urllib.error.HTTPError as error: response=error
        except Exception as error:
            self.send_error(502,str(error)); return
        status=response.code
        body=response.read()
        response_headers={k.lower():v for k,v in response.headers.items()}
        content_type=response_headers.get('content-type','application/octet-stream')
        if response_headers.get('content-encoding')=='gzip':
            body=gzip.decompress(body); response_headers.pop('content-encoding',None)
        if status==200 and protocol in ['cargo','pypi','nuget','dart','composer','conan','swift'] and ('json' in content_type or body.startswith(b'{')):
            try:
                body=json.dumps(self.rewrite(protocol,json.loads(body),target),separators=(',',':')).encode()
                response_headers.pop('etag',None)
            except (ValueError,UnicodeError): pass
        if status==200 and protocol=='pypi' and 'html' in content_type:
            def link(match):
                absolute=urllib.parse.urljoin(target,match[2].replace('&amp;','&'))
                return match[1]+self.bind(protocol,absolute).replace('&','&amp;')+match[3]
            body=re.sub(r'''(?i)(href\s*=\s*["'])([^"']+)(["'])''',link,body.decode()).encode()
        if protocol=='swift' and 'link' in response_headers:
            response_headers['link']=re.sub(r'<([^>]+)>',lambda m:'<'+self.bind(protocol,urllib.parse.urljoin(target,m[1]))+'>',response_headers['link'])
        with self.guard:
            self.counts[protocol]=self.counts.get(protocol,0)+1
            with self.logpath.open('a') as log:
                log.write(json.dumps({'protocol':protocol,'path':parsed.path,'status':status,'bytes':len(body),'upstream_host':urllib.parse.urlsplit(target).hostname})+'\n')
        self.send_response(status)
        for name in ['content-type','content-encoding','content-version','content-disposition','digest','link','docker-content-digest','docker-distribution-api-version','www-authenticate','x-conan-server-capabilities']:
            if name in response_headers: self.send_header(name,response_headers[name])
        self.send_header('Content-Length',len(body)); self.end_headers()
        try: self.wfile.write(body)
        except (BrokenPipeError,ConnectionResetError): pass

class TLSFront(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def do_HEAD(self): self.forward('HEAD')
    def do_GET(self): self.forward('GET')
    def forward(self, method):
        headers={'Host':'127.0.0.1:28443','X-Forwarded-Proto':'https'}
        for name in ['Accept','Authorization','User-Agent']:
            if self.headers.get(name): headers[name]=self.headers[name]
        try: response=urllib.request.urlopen(urllib.request.Request(URL+self.path,headers=headers,method=method),timeout=180)
        except urllib.error.HTTPError as error: response=error
        except OSError:
            self.send_error(502, 'Validation server connection closed'); return
        body=response.read()
        self.send_response(response.code)
        for key,value in response.headers.items():
            if key.lower() not in ['transfer-encoding','connection','server','date']: self.send_header(key,value)
        try:
            self.end_headers()
            if method!='HEAD': self.wfile.write(body)
        except OSError:
            return  # Native clients may cancel speculative parallel metadata requests.

class Runner:
    def __init__(self, args):
        self.args=args
        self.run=ROOT/'evidence/all-proxy'/time.strftime('run-%Y%m%d-%H%M%S')
        self.run.mkdir(parents=True)
        self.env=os.environ.copy()
        home=pathlib.Path.home()
        self.env['PATH']=f'{home}/.cargo/bin:{home}/.local/bin:{home}/.nvm/versions/node/v22.20.0/bin:{home}/.sdkman/candidates/maven/current/bin:{home}/.sdkman/candidates/java/current/bin:'+self.env['PATH']
        self.env.update(TEAQL_REGISTRY_SERVICE_CORE_DATABASE_URL='postgresql://proxy_test:proxy_test@127.0.0.1:25441/proxy_test',TEAQL_REGISTRY_SERVICE_CORE_DATABASE_USER='proxy_test',TEAQL_REGISTRY_SERVICE_CORE_DATABASE_PASSWORD='proxy_test',PROXY_TEST_BLOB_ROOT=str(self.run/'data/blobs'),PROXY_TEST_UPSTREAM_ROOT=GATEWAY,TEAQL_PROXY_SUMDB_BASE_URL=GATEWAY+'/sumdb',RUST_MIN_STACK='33554432')
        self.binary=pathlib.Path(args.binary)
        self.server=None; self.gateway=None; self.tlsfront=None; self.results={}
    def command(self, label, command, cwd=None, env=None, timeout=240):
        output=self.run/(label+'.log')
        started=time.time()
        with output.open('w') as log:
            proc=subprocess.run([str(x) for x in command],cwd=cwd,env=env or self.env,stdout=log,stderr=subprocess.STDOUT,timeout=timeout)
        if proc.returncode:
            print(output.read_text(errors='replace')[-4500:],flush=True)
            raise RuntimeError(f'{label} failed ({proc.returncode}): {output}')
        print(f'PASS {label} ({time.time()-started:.1f}s)',flush=True)
        return output
    def start(self):
        if not self.args.reuse_db:
            subprocess.run(['docker','rm','-f','teaql-all-proxy-pg'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            self.command('postgres-start',['docker','run','-d','--name','teaql-all-proxy-pg','-p','127.0.0.1:25441:5432','-e','POSTGRES_USER=proxy_test','-e','POSTGRES_PASSWORD=proxy_test','-e','POSTGRES_DB=proxy_test','postgres:16-alpine'])
        for _ in range(60):
            proc=subprocess.run(['docker','exec','teaql-all-proxy-pg','pg_isready','-h','127.0.0.1','-U','proxy_test'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            if proc.returncode==0: break
            time.sleep(1)
        Gateway.logpath=self.run/'upstream.jsonl'; Gateway.counts={}; Gateway.mappings={}
        self.gateway=http.server.ThreadingHTTPServer(('127.0.0.1',28445),Gateway)
        threading.Thread(target=self.gateway.serve_forever,daemon=True).start()
        if 'nuget' in self.args.protocols:
            certs=self.run/'certs'; certs.mkdir()
            self.command('tls-cert',['openssl','req','-x509','-newkey','rsa:2048','-nodes','-keyout',certs/'key.pem','-out',certs/'ca.crt','-days','1','-subj','/CN=127.0.0.1','-addext','subjectAltName=IP:127.0.0.1','-addext','basicConstraints=critical,CA:TRUE'])
            cert_hash=subprocess.check_output(['openssl','x509','-hash','-noout','-in',str(certs/'ca.crt')],env=self.env,text=True).strip()
            (certs/(cert_hash+'.0')).symlink_to('ca.crt')
            context=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER); context.load_cert_chain(certs/'ca.crt',certs/'key.pem')
            self.tlsfront=http.server.ThreadingHTTPServer(('127.0.0.1',28443),TLSFront)
            self.tlsfront.socket=context.wrap_socket(self.tlsfront.socket,server_side=True)
            threading.Thread(target=self.tlsfront.serve_forever,daemon=True).start()
        self.start_server('online')
    def start_server(self,phase):
        log=(self.run/f'server-{phase}.log').open('w')
        self.server=subprocess.Popen([str(self.binary)],stdout=log,stderr=subprocess.STDOUT,env=self.env)
        for _ in range(120):
            if self.server.poll() is not None: raise RuntimeError((self.run/f'server-{phase}.log').read_text()[-3500:])
            try:
                urllib.request.urlopen(URL+'/help',timeout=2).read(); return
            except (OSError,urllib.error.URLError): time.sleep(1)
        raise RuntimeError('Server did not start')
    def stop_server(self):
        if self.server:
            self.server.terminate(); self.server.wait(timeout=30); self.server=None
    def client_env(self,work):
        env=self.env.copy()
        env['RUSTUP_HOME']=os.environ.get('RUSTUP_HOME',str(pathlib.Path.home()/'.rustup'))
        env.update(HOME=str(work/'home'),XDG_CACHE_HOME=str(work/'cache'),HTTP_PROXY='http://127.0.0.1:9',HTTPS_PROXY='http://127.0.0.1:9',ALL_PROXY='http://127.0.0.1:9',NO_PROXY='127.0.0.1,localhost',http_proxy='http://127.0.0.1:9',https_proxy='http://127.0.0.1:9',no_proxy='127.0.0.1,localhost')
        (work/'home').mkdir(parents=True,exist_ok=True)
        return env
    def container(self,image,work,args,extra=None):
        command=['docker','run','--rm','--network','host','--user',f'{os.getuid()}:{os.getgid()}','-v',f'{work}:/work','-w','/work']
        for key,value in {'HOME':'/work/home','HTTP_PROXY':'http://127.0.0.1:9','HTTPS_PROXY':'http://127.0.0.1:9','NO_PROXY':'localhost,127.0.0.1',**(extra or {})}.items(): command+=['-e',f'{key}={value}']
        return command+[image]+args
    def prepare(self):
        # Produce real Swift/Conan upstream artifacts using their native clients.
        if 'swift' in self.args.protocols:
            work=self.run/'fixtures/swift'; (work/'Sources/ProxyFixture').mkdir(parents=True)
            (work/'Package.swift').write_text('// swift-tools-version: 5.9\nimport PackageDescription\nlet package = Package(name:"ProxyFixture",products:[.library(name:"ProxyFixture",targets:["ProxyFixture"])],targets:[.target(name:"ProxyFixture")])\n')
            (work/'Sources/ProxyFixture/Probe.swift').write_text('public let proxyAnswer = 42\n')
            self.command('swift-upstream-publish',['swift','package-registry','publish','fixture.proxy','1.0.0','--url',URL+'/repository/fixture-swift/swift','--allow-insecure-http'],cwd=work)
        if 'conan' in self.args.protocols:
            work=self.run/'fixtures/conan'; work.mkdir(parents=True)
            (work/'conanfile.py').write_text('from conan import ConanFile\nfrom conan.tools.files import copy\nclass Fixture(ConanFile):\n name="proxyfixture"\n version="1.0"\n package_type="header-library"\n exports_sources="proxy.h"\n def package(self):\n  copy(self,"proxy.h",src=self.source_folder,dst=self.package_folder+"/include")\n def package_id(self):\n  self.info.clear()\n')
            (work/'proxy.h').write_text('#define PROXY_ANSWER 42\n')
            env=self.env.copy(); env['CONAN_HOME']=str(work/'conan-home')
            conan=ROOT/'.proxy-client-venv/bin/conan'
            self.command('conan-profile',[conan,'profile','detect','--force'],env=env)
            self.command('conan-create',[conan,'create',work],env=env)
            self.command('conan-upstream-add',[conan,'remote','add','fixture',URL+'/repository/fixture-conan/conan','--insecure'],env=env)
            self.command('conan-upstream-publish',[conan,'upload','proxyfixture/1.0','-r','fixture','--confirm'],env=env)
        if 'hex' in self.args.protocols:
            mix=self.run/'toolchain/mix'; mix.mkdir(parents=True)
            self.command('hex-bootstrap',['docker','run','--rm','--network','host','-v',f'{mix}:/root/.mix','elixir:1.18-alpine','mix','local.hex','--force'])
        if 'raw' in self.args.protocols:
            urllib.request.urlopen(urllib.request.Request(URL+'/repository/fixture-raw/probe.txt',data=b'TeaQL raw proxy native fixture\n',method='PUT')).read()
    def consume(self,protocol,phase):
        work=self.run/phase/protocol; work.mkdir(parents=True)
        env=self.client_env(work); label=phase+'-'+protocol
        base=URL+'/repository/proxy-'+protocol
        if protocol=='cargo':
            (work/'src').mkdir(); (work/'src/main.rs').write_text('fn main(){assert_eq!(itoa::Buffer::new().format(42),"42");}\n')
            (work/'Cargo.toml').write_text('[package]\nname="proxy-consumer"\nversion="0.1.0"\nedition="2021"\n[workspace]\n[dependencies]\nitoa="=1.0.15"\n')
            (work/'cargo').mkdir(); (work/'cargo/config.toml').write_text(f'[source.crates-io]\nreplace-with="proxy"\n[source.proxy]\nregistry="sparse+{base}/cargo/index/"\n')
            env.update(CARGO_HOME=str(work/'cargo'),CARGO_TARGET_DIR=str(work/'target'))
            self.command(label,['cargo','run','--manifest-path',work/'Cargo.toml'],cwd=work,env=env)
        elif protocol=='pypi':
            self.command(label,['python3','-m','pip','--isolated','install','--no-cache-dir','--disable-pip-version-check','--index-url',base+'/simple/','--trusted-host','127.0.0.1','--target',work/'packages','packaging==24.2'],env=env)
            env['PYTHONPATH']=str(work/'packages'); self.command(label+'-use',['python3','-c','import packaging; assert packaging.__version__ == "24.2"'],env=env)
        elif protocol=='gomod':
            (work/'go.mod').write_text('module example.com/proxyconsumer\ngo 1.20\nrequire rsc.io/quote v1.5.2\n')
            (work/'main.go').write_text('package main\nimport("fmt";"rsc.io/quote")\nfunc main(){fmt.Println(quote.Hello())}\n')
            env.update(GOPROXY=base+'/gomod',GOSUMDB='sum.golang.org',GOMODCACHE=str(work/'modcache'),GOCACHE=str(work/'gocache'),GOTOOLCHAIN='local',GOFLAGS='-mod=mod')
            self.command(label,['go','run','.'],cwd=work,env=env)
        elif protocol=='nuget':
            base='https://127.0.0.1:28443/repository/proxy-nuget'
            env.update(SSL_CERT_FILE=str(self.run/'certs/ca.crt'),SSL_CERT_DIR=str(self.run/'certs')+':/etc/ssl/certs')
            (work/'consumer.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net8.0</TargetFramework><OutputType>Exe</OutputType><NuGetAudit>false</NuGetAudit></PropertyGroup><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3"/></ItemGroup></Project>')
            (work/'Program.cs').write_text('System.Console.WriteLine(Newtonsoft.Json.JsonConvert.SerializeObject(new { Answer = 42 }));')
            (work/'NuGet.Config').write_text(f'<configuration><packageSources><clear/><add key="proxy" value="{base}/v3/index.json" allowInsecureConnections="true"/></packageSources></configuration>')
            env.update(NUGET_PACKAGES=str(work/'packages'),NUGET_HTTP_CACHE_PATH=str(work/'httpcache'),DOTNET_CLI_HOME=str(work/'home'),DOTNET_SKIP_FIRST_TIME_EXPERIENCE='1',DOTNET_CLI_TELEMETRY_OPTOUT='1',DOTNET_NUGET_SIGNATURE_VERIFICATION='true',DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE='true')
            self.command(label,['dotnet','restore','--configfile',work/'NuGet.Config','--disable-parallel'],cwd=work,env=env)
            self.command(label+'-use',['dotnet','run','--no-restore'],cwd=work,env=env)
        elif protocol=='docker':
            image='127.0.0.1:28441/proxy-docker/library/busybox:1.37.0'
            subprocess.run(['docker','image','rm',image],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            pull_log=self.command(label,['docker','pull',image],timeout=300)
            assert 'Pull complete' in pull_log.read_text(), 'Docker reused local layers instead of downloading them'
            self.command(label+'-use',['docker','run','--rm',image,'echo','proxy-native-42'])
            self.command(label+'-remove',['docker','image','rm',image])
        elif protocol=='swift':
            (work/'Sources/Consumer').mkdir(parents=True)
            (work/'Package.swift').write_text('// swift-tools-version: 5.9\nimport PackageDescription\nlet package=Package(name:"Consumer",dependencies:[.package(id:"fixture.proxy",exact:"1.0.0")],targets:[.executableTarget(name:"Consumer",dependencies:[.product(name:"ProxyFixture",package:"fixture.proxy")])])\n')
            (work/'Sources/Consumer/main.swift').write_text('import ProxyFixture\nprecondition(proxyAnswer == 42)\nprint(proxyAnswer)\n')
            self.command(label+'-config',['swift','package-registry','set','--allow-insecure-http',base+'/swift'],cwd=work,env=env)
            self.command(label,['swift','package','--cache-path',work/'cache','--security-path',work/'security','resolve'],cwd=work,env=env)
            self.command(label+'-use',['swift','run','--cache-path',work/'cache','--security-path',work/'security','Consumer'],cwd=work,env=env,timeout=300)
        elif protocol=='dart':
            (work/'pubspec.yaml').write_text('name: proxy_consumer\nenvironment:\n  sdk: ">=3.0.0 <4.0.0"\ndependencies:\n  collection: 1.19.1\n')
            (work/'bin').mkdir(); (work/'bin/main.dart').write_text("import 'package:collection/collection.dart';\nvoid main(){if(!const IterableEquality<int>().equals([42],[42])) throw StateError('dependency failed'); print(42);}\n")
            extra={'PUB_HOSTED_URL':base+'/dart','PUB_CACHE':'/work/pubcache'}
            self.command(label,self.container('dart:stable',work,['dart','pub','get'],extra))
            self.command(label+'-use',self.container('dart:stable',work,['dart','run','bin/main.dart'],extra))
        elif protocol=='rubygems':
            extra={'GEM_HOME':'/work/gems','GEM_PATH':'/work/gems'}
            self.command(label,self.container('ruby:3.4-alpine',work,['gem','install','rake','--version','13.2.1','--no-document','--force','--clear-sources','--source',base+'/rubygems/'],extra))
            self.command(label+'-use',self.container('ruby:3.4-alpine',work,['ruby','-e','require "rake"; abort unless Rake::VERSION == "13.2.1"; puts Rake::VERSION'],extra))
        elif protocol=='composer':
            (work/'composer.json').write_text(json.dumps({'name':'fixture/consumer','require':{'psr/log':'3.0.2'},'repositories':[{'type':'composer','url':base+'/composer'},{'packagist.org':False}],'config':{'secure-http':False,'allow-plugins':False}}))
            extra={'COMPOSER_HOME':'/work/home','COMPOSER_CACHE_DIR':'/work/cache','COMPOSER_ALLOW_SUPERUSER':'1'}
            self.command(label,self.container('composer:2',work,['composer','install','--no-dev','--no-interaction','--prefer-dist','--no-plugins','--no-scripts'],extra))
            self.command(label+'-use',self.container('composer:2',work,['php','-r',"require 'vendor/autoload.php'; if (!interface_exists('Psr\\Log\\LoggerInterface')) exit(1); echo '42';"],extra))
        elif protocol=='conan':
            env['CONAN_HOME']=str(work/'conan-home'); conan=ROOT/'.proxy-client-venv/bin/conan'
            self.command(label+'-profile',[conan,'profile','detect','--force'],env=env)
            self.command(label+'-remote',[conan,'remote','add','proxy',base+'/conan','--insecure'],env=env)
            self.command(label,[conan,'install','--requires=proxyfixture/1.0','-r','proxy','--build=never'],cwd=work,env=env)
            headers=list((work/'conan-home').glob('p/**/p/include/proxy.h')); assert headers, 'Conan did not download the binary package header'
            assert 'PROXY_ANSWER 42' in headers[0].read_text()
        elif protocol=='hex':
            (work/'mix.exs').write_text('defmodule ProxyConsumer.MixProject do\n use Mix.Project\n def project, do: [app: :proxy_consumer, version: "0.1.0", deps: [{:jason, "1.4.4"}]]\n def application, do: []\nend\n')
            extra={'HEX_MIRROR':base+'/hex/repo','HEX_MIRROR_URL':base+'/hex/repo','HEX_HOME':'/work/hexhome','MIX_HOME':'/toolchainmix','MIX_DEPS_PATH':'/work/deps','MIX_BUILD_PATH':'/work/build'}
            def hexcmd(args):
                command=self.container('elixir:1.18-alpine',work,args,extra)
                command[command.index('elixir:1.18-alpine'):command.index('elixir:1.18-alpine')]=['-v',f'{self.run}/toolchain/mix:/toolchainmix:ro']
                return command
            self.command(label,hexcmd(['mix','deps.get']))
            self.command(label+'-use',hexcmd(['mix','run','-e','IO.puts(Jason.encode!(%{answer: 42}))']))
        elif protocol=='raw':
            self.command(label,['curl','--noproxy','*','-fsS',base+'/probe.txt','-o',work/'probe.txt'],env=env)
            assert (work/'probe.txt').read_bytes()==b'TeaQL raw proxy native fixture\n'
        else: raise RuntimeError('Unknown protocol '+protocol)
    def run_all(self):
        try:
            self.start()
            self.prepare()
            for phase in ['cold','cached','offline']:
                if phase=='offline':
                    self.gateway.shutdown(); self.gateway.server_close(); self.gateway=None
                    self.stop_server(); self.start_server('offline')
                for protocol in self.args.protocols:
                    if phase!='cold' and self.results.get(protocol,{}).get('cold',{}).get('status')!='PASS': continue
                    before=Gateway.counts.get(protocol,0)
                    try:
                        self.consume(protocol,phase)
                        after=Gateway.counts.get(protocol,0)
                        if phase=='cold': assert after>before, f'{protocol}: client never reached upstream'
                        else: assert after==before, f'{protocol}: unexpected upstream traffic in {phase}: {after-before}'
                        self.results.setdefault(protocol,{})[phase]={'status':'PASS','upstream_requests':after-before}
                    except Exception as error:
                        self.results.setdefault(protocol,{})[phase]={'status':'FAIL','error':str(error)}
                        print(f'FAIL {phase}-{protocol}: {error}',flush=True)
                    self.save()
            failed=[(p,phase) for p,phases in self.results.items() for phase,result in phases.items() if result['status']!='PASS']
            if failed: raise RuntimeError(f'Failed native proxy phases: {failed}; evidence: {self.run}')
            print('ALL_PROXY_NATIVE_PHASES_PASSED '+str(self.run),flush=True)
        finally:
            self.stop_server()
            if self.gateway: self.gateway.shutdown(); self.gateway.server_close()
            if self.tlsfront: self.tlsfront.shutdown(); self.tlsfront.server_close()
    def save(self):
        (self.run/'summary.json').write_text(json.dumps(self.results,indent=2)+'\n')
        (ROOT/'evidence/all-proxy/latest-run.txt').write_text(str(self.run)+'\n')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--protocols',type=lambda s:s.split(','),default=ALL)
    parser.add_argument('--binary',default='/tmp/teaql-regist-proxy-target/debug/examples/proxy_validation_server')
    parser.add_argument('--reuse-db',action='store_true')
    args=parser.parse_args()
    Runner(args).run_all()
