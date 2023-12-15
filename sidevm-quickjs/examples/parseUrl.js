console.log = Sidevm.inspect;
const url1 = new URL('http://example.com/foo/bar?baz=qux&k2=v2#quux');
console.log('url1:', url1);
console.log('url1.searchParams():', url1.searchParams());
const url2 = new URL('http://user:pass@localhost:8080/foo/bar?baz=qux#quux');
console.log('url2:', url2);