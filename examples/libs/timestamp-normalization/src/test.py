#!/usr/bin/env python3
# coding: utf-8

import sys, ctypes, os, json
from ctypes import c_int32, c_char_p, c_void_p

def py_check(path):
    prefix = {'win32': ''}.get(sys.platform, 'lib')
    extension = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')

    __location__ = os.path.realpath(
        os.path.join(os.getcwd(), os.path.dirname(__file__)))
    lib = ctypes.cdll.LoadLibrary(os.path.join(__location__, prefix + "timestamp_normalization" + extension))

    lib.check.argtypes = (c_char_p,)
    lib.check.restype = c_void_p

    lib.check_free.argtypes = (c_void_p, )

    ptr = lib.check(path.encode('utf-8'))
    try:
        return json.loads(ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8'))
    finally:
        lib.check_free(ptr)

if __name__ == "__main__":
    if len(sys.argv) == 1:
        print("need flv path.")
    else:
        ret = py_check(sys.argv[1])
        print(ret)