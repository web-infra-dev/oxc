var msg = "hello";
console.log(msg);

// for env detection
globalThis = 'abc';

// for globals detection
DefaultFoo = 'defaultReadable';
ReadFoo = 'readable';
console.log(ReadFoo);
WriteBar = 'writeable';
NonBaz = 'always-off';