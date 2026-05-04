import struct
dbf = "data/2020/tiger/tracts/tl_2020_55_tract/tl_2020_55_tract.dbf"
with open(dbf, 'rb') as f: d = f.read()
n_rec = struct.unpack_from('<I', d, 4)[0]
hdr = struct.unpack_from('<H', d, 8)[0]
rec_sz = struct.unpack_from('<H', d, 10)[0]
fields = []; pos = 32
while pos + 32 <= len(d) and d[pos] != 0x0D:
    name = d[pos:pos+11].rstrip(b'\x00').decode('ascii','replace').strip()
    fields.append((name, d[pos+16])); pos += 32
aland_vals = []
for r in range(n_rec):
    s = hdr + r * rec_sz
    if d[s] == 0x2A: continue
    off = 1
    for nm, ln in fields:
        if nm == 'ALAND':
            v = d[s+off:s+off+ln].decode('ascii','replace').strip()
            try: aland_vals.append(int(v))
            except: pass
        off += ln
print(f"WI tracts: {len(aland_vals)}")
print(f"ALAND range: {min(aland_vals):,} - {max(aland_vals):,} m2")
print(f"Max in hectares (div 10000): {max(aland_vals)//10000:,}")
print(f"i32 max: {2**31-1:,}")
print(f"Hectare overflows i32? {max(aland_vals)//10000 > 2**31-1}")
