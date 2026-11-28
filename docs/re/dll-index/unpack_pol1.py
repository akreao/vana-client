"""Rebuild FFXiMain.dll with .text decompressed from the POL1 section (static copy for analysis).
usage: unpack_pol1.py FFXiMain.dll FFXiMain.unpacked.dll
The entry stub (in POL1, after the compressed bytes) gives every number: `mov eax, <out len>`, `push <comp len>`,
and the final `jmp <OEP>`. Stream: flag byte MSB-first, 1 = literal, 0 = 2-byte backref (off = 12 bits,
len = hi nibble + 3, off 0 = end)."""
import sys, struct, pefile
src, out = sys.argv[1], sys.argv[2]
pe = pefile.PE(src)
data = bytearray(open(src, 'rb').read())
base = pe.OPTIONAL_HEADER.ImageBase
pol = next(s for s in pe.sections if s.Name.rstrip(b'\0') == b'POL1')
text = next(s for s in pe.sections if s.Name.rstrip(b'\0') == b'.text')
ep = pe.OPTIONAL_HEADER.AddressOfEntryPoint
stub = pe.get_data(ep, 0xa0)
assert stub[:5] == bytes.fromhex('807c240801'), 'unknown entry stub'
i = stub.index(b'\x6a\x00\xb8')            # push 0; mov eax, out_len
out_len = struct.unpack_from('<I', stub, i + 3)[0]
j = stub.index(b'\x50\x68', i)             # push eax; push comp_len
comp_len = struct.unpack_from('<I', stub, j + 2)[0]
k = stub.index(b'\x83\xc4\x08\x61\xe9')    # add esp,8; popal; jmp OEP
oep = ep + k + 9 + struct.unpack_from('<i', stub, k + 5)[0]
assert comp_len == ep - pol.VirtualAddress and out_len == text.Misc_VirtualSize, (hex(comp_len), hex(out_len))
comp = pol.get_data()[:comp_len]
o = bytearray(); i = 0
while True:
    flags = comp[i]; i += 1; done = False
    for _ in range(8):
        lit = flags & 0x80; flags = (flags << 1) & 0xff
        if lit:
            o.append(comp[i]); i += 1
        else:
            b0, b1 = comp[i], comp[i+1]
            off = ((b0 << 8) | b1) & 0xfff
            if off == 0: done = True; break
            i += 2
            n = (b0 >> 4) + 3
            for _ in range(n): o.append(o[-off])
    if done: break
assert len(o) == out_len, (hex(len(o)), hex(out_len))
print('decompressed', hex(len(o)), 'OEP', hex(base + oep))
fa = pe.OPTIONAL_HEADER.FileAlignment
data += b'\0' * ((-len(data)) % fa)
raw_off = len(data)
blob = bytes(o) + b'\0' * ((-len(o)) % fa)
data += blob
hdr = text.get_file_offset()
struct.pack_into('<I', data, hdr + 16, len(blob))
struct.pack_into('<I', data, hdr + 20, raw_off)
struct.pack_into('<I', data, pe.OPTIONAL_HEADER.get_file_offset() + 16, oep)
open(out, 'wb').write(data)
