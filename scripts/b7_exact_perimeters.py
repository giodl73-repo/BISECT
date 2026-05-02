"""
Compute exact TIGER polygon perimeters for focal states.

Reads .shp files directly (no shapefile library needed),
projects WGS84 -> EPSG:5070 (Albers Equal Area, CONUS) using pyproj,
computes polygon perimeter in metres.

Returns: {geoid: total_perimeter_m} for each state.

Then computes:
  ext_perim(v) = total_perim(v) - sum(shared boundary weights with all neighbors)
  PP(S) = 4*pi * area(S) / perimeter(S)^2  [dimensionally consistent in m]
"""
import struct, json, math, os
import pyproj

# WGS84 -> EPSG:5070 (Albers Equal Area, continental US)
TRANSFORMER = pyproj.Transformer.from_crs("EPSG:4269", "EPSG:5070", always_xy=True)

def read_shp_polygons(shp_path):
    """Read polygon records from TIGER .shp file. Returns {geoid: [(x,y),...]}."""
    # Need to read .dbf to get GEOIDs
    dbf_path = shp_path.replace('.shp', '.dbf')
    geoids = read_dbf_geoids(dbf_path)

    polygons = {}
    with open(shp_path, 'rb') as f:
        f.seek(100)  # skip file header
        record_idx = 0
        while True:
            rec_header = f.read(8)
            if len(rec_header) < 8: break
            content_length = struct.unpack('>I', rec_header[4:])[0] * 2  # in bytes
            content = f.read(content_length)
            if len(content) < 4: break
            shape_type = struct.unpack_from('<i', content, 0)[0]
            if shape_type == 5:  # Polygon
                # bbox: 32 bytes at offset 4
                num_parts = struct.unpack_from('<i', content, 36)[0]
                num_points = struct.unpack_from('<i', content, 40)[0]
                parts_offset = 44
                points_offset = parts_offset + num_parts * 4
                parts = [struct.unpack_from('<i', content, parts_offset + i*4)[0]
                        for i in range(num_parts)]
                parts.append(num_points)
                coords = [(struct.unpack_from('<d', content, points_offset + i*16)[0],
                           struct.unpack_from('<d', content, points_offset + i*16 + 8)[0])
                          for i in range(num_points)]
                # First ring (exterior)
                ext_ring = coords[parts[0]:parts[1]]
                geoid = geoids[record_idx] if record_idx < len(geoids) else ''
                if geoid:
                    polygons[geoid] = ext_ring
            record_idx += 1
    return polygons

def read_dbf_geoids(dbf_path):
    """Extract GEOID field from .dbf file, in record order."""
    geoids = []
    try:
        with open(dbf_path, 'rb') as f:
            header = f.read(32)
            n_records = struct.unpack_from('<I', header, 4)[0]
            header_size = struct.unpack_from('<H', header, 8)[0]
            record_size = struct.unpack_from('<H', header, 10)[0]
            fields = []
            f.seek(32)
            while True:
                fd = f.read(32)
                if not fd or fd[0] == 0x0D: break
                name = fd[:11].rstrip(b'\x00').decode('ascii','replace')
                length = fd[16]
                fields.append((name, length))
            f.seek(header_size)
            geoid_field = next((i for i,(n,_) in enumerate(fields)
                               if 'GEOID' in n.upper()), None)
            for _ in range(n_records):
                row = f.read(record_size)
                if not row or row[0] == 0x2A: continue
                pos = 1
                vals = []
                for name, length in fields:
                    vals.append(row[pos:pos+length].decode('ascii','replace').strip())
                    pos += length
                if geoid_field is not None:
                    geoids.append(vals[geoid_field][-11:])  # last 11 chars
    except Exception as e:
        pass
    return geoids

def polygon_perimeter_m(coords_wgs84):
    """Compute polygon perimeter in metres after projecting to EPSG:5070."""
    if len(coords_wgs84) < 2: return 0.0
    # Project all coords
    xs = [c[0] for c in coords_wgs84]
    ys = [c[1] for c in coords_wgs84]
    px, py = TRANSFORMER.transform(xs, ys)
    perim = 0.0
    for i in range(len(px) - 1):
        dx = px[i+1] - px[i]
        dy = py[i+1] - py[i]
        perim += math.sqrt(dx*dx + dy*dy)
    return perim

def load_adj(path):
    with open(path,'rb') as f: data=f.read()
    pos=4; _=struct.unpack_from('<I',data,pos)[0]; pos+=4
    n=struct.unpack_from('<I',data,pos)[0]; pos+=4; pos+=4
    vw=[]
    for _ in range(n): vw.append(struct.unpack_from('<q',data,pos)[0]); pos+=8
    adj=[]
    for _ in range(n):
        nb=struct.unpack_from('<I',data,pos)[0]; pos+=4
        nbrs=[struct.unpack_from('<I',data,pos+i*4)[0] for i in range(nb)]; pos+=nb*4
        adj.append(nbrs)
    nw=struct.unpack_from('<I',data,pos)[0]; pos+=4
    ew={}
    for _ in range(nw):
        u=struct.unpack_from('<I',data,pos)[0]; pos+=4
        v=struct.unpack_from('<I',data,pos)[0]; pos+=4
        w=struct.unpack_from('<d',data,pos)[0]; pos+=8
        ew[(min(u,v),max(u,v))]=w
    return n, adj, ew, vw

FIPS = {"wi":"55","pa":"42","nc":"37","ga":"13","mi":"26",
        "tn":"47","mn":"27","tx":"48"}

def get_geometry(code, year="2020"):
    """Returns (vertex_areas_m2, vertex_ext_perims_m) aligned to adj.bin index."""
    fips = FIPS[code]
    tiger_dir = f"data/{year}/tiger/tracts/tl_{year}_{fips}_tract"
    shp_path = f"{tiger_dir}/tl_{year}_{fips}_tract.shp"
    adj_path = f"outputs/V3/data/{year}/adjacency/{code}_adjacency_{year}.adj.bin"
    geoid_path = f"outputs/V3/data/{year}/adjacency/{code}_adjacency_{year}_geoids.json"

    if not os.path.exists(shp_path):
        print(f"  {code.upper()}: TIGER shp not found at {shp_path}")
        return None, None

    print(f"  {code.upper()}: loading TIGER polygons...", end=' ')
    polygons = read_shp_polygons(shp_path)
    print(f"{len(polygons)} polygons")

    # Build perimeter map
    perim_map = {}
    for geoid, coords in polygons.items():
        perim_map[geoid] = polygon_perimeter_m(coords)

    # Load adj.bin and geoid map
    n, adj, ew, vw = load_adj(adj_path)
    idx2geoid = json.load(open(geoid_path))

    # Load ALAND from dbf
    dbf_path = f"{tiger_dir}/tl_{year}_{fips}_tract.dbf"
    aland_map = {}
    try:
        with open(dbf_path,'rb') as f:
            header=f.read(32)
            n_rec=struct.unpack_from('<I',header,4)[0]
            hdr_size=struct.unpack_from('<H',header,8)[0]
            rec_size=struct.unpack_from('<H',header,10)[0]
            fields=[]
            f.seek(32)
            while True:
                fd=f.read(32)
                if not fd or fd[0]==0x0D: break
                name=fd[:11].rstrip(b'\x00').decode('ascii','replace')
                length=fd[16]; fields.append((name,length))
            f.seek(hdr_size)
            for _ in range(n_rec):
                row=f.read(rec_size)
                if not row or row[0]==0x2A: continue
                pos=1; vals={}
                for fname,flen in fields:
                    vals[fname]=row[pos:pos+flen].decode('ascii','replace').strip(); pos+=flen
                geoid=vals.get('GEOID','')[-11:]
                aland=float(vals.get('ALAND','0') or '0')
                if geoid: aland_map[geoid]=aland
    except Exception as e:
        print(f"  ALAND load error: {e}")

    # Build aligned arrays
    vertex_areas = []
    vertex_ext_perims = []
    missing = 0

    for i in range(n):
        geoid = idx2geoid.get(str(i), '')
        area = float(aland_map.get(geoid, 0))
        total_perim = perim_map.get(geoid, 0)
        shared = sum(ew.get((min(i,j),max(i,j)),0) for j in adj[i])
        ext_perim = max(total_perim - shared, 0)

        vertex_areas.append(area)
        vertex_ext_perims.append(ext_perim)
        if area == 0 or total_perim == 0:
            missing += 1

    print(f"  {code.upper()}: {n} tracts, {missing} missing geometry "
          f"({100*missing/n:.0f}%)")
    return vertex_areas, vertex_ext_perims

def compute_gmpp_exact(left_set, right_set, adj, ew, vertex_areas, vertex_ext_perims):
    """GMPP with exact perimeters. PP in [0,1] iff inputs are correct."""
    def subgraph_pp(verts):
        A = sum(vertex_areas[v] for v in verts)
        if A <= 0: return 0
        ext = sum(vertex_ext_perims[v] for v in verts)
        seen = set()
        boundary = 0
        for u in verts:
            for v in adj[u]:
                if v not in verts:
                    key = (min(u,v),max(u,v))
                    if key not in seen:
                        seen.add(key); boundary += ew.get(key,0)
        P = ext + boundary
        if P <= 0: return 0
        return min(4 * math.pi * A / (P * P), 1.0)  # clamp to [0,1]

    pp_l = subgraph_pp(left_set)
    pp_r = subgraph_pp(right_set)
    return math.sqrt(pp_l * pp_r), pp_l, pp_r

if __name__ == '__main__':
    print("Computing exact TIGER perimeters for focal states...\n")
    for code in ["wi","pa","nc","ga","mi","tn","mn"]:
        areas, ext_perims = get_geometry(code)
        if areas:
            pos_areas = sorted(a for a in areas if a > 0)
            pos_ext   = sorted(p for p in ext_perims if p > 0)
            if pos_areas and pos_ext:
                median_area = pos_areas[len(pos_areas)//2]
                median_ext  = pos_ext[len(pos_ext)//2]
                r = math.sqrt(median_area/math.pi)
                circ_perim = 2*math.pi*r
                pp_circ = 4*math.pi*median_area / (circ_perim**2)
                print(f"  {code.upper()} median tract: area={median_area/1e6:.2f}km2 "
                      f"ext_perim={median_ext/1000:.1f}km  PP_circle_approx={pp_circ:.3f}")
    print("\nDone. Import get_geometry() from this module for GMPP analysis.")
