import sqlite3, time, sys

db = sys.argv[1]
con = sqlite3.connect(db)
cur = con.cursor()

# index check on the join queries the app runs
for t in ("project_frameworks","project_package_managers","project_languages"):
    plan = cur.execute(f"EXPLAIN QUERY PLAN SELECT * FROM {t} WHERE project_id=? ORDER BY id", (1,)).fetchall()
    print(t, "->", plan[0][-1] if plan else "?")

# replay the app's N+1: 1 list query, then 3 per project, serial
t0 = time.perf_counter()
ids = [r[0] for r in cur.execute("SELECT id FROM projects").fetchall()]
nq = 1
for pid in ids:
    for t in ("project_frameworks","project_package_managers","project_languages"):
        cur.execute(f"SELECT * FROM {t} WHERE project_id=? ORDER BY id", (pid,)).fetchall()
        nq += 1
dt = (time.perf_counter()-t0)*1000
print(f"\nprojects={len(ids)}  queries={nq}  total={dt:.1f}ms  per_query={dt/nq:.3f}ms")

# detail path: get_by_id = 1 + 3
t0 = time.perf_counter()
cur.execute("SELECT * FROM projects WHERE id=?", (ids[len(ids)//2],)).fetchall()
for t in ("project_frameworks","project_package_managers","project_languages"):
    cur.execute(f"SELECT * FROM {t} WHERE project_id=? ORDER BY id", (ids[len(ids)//2],)).fetchall()
print(f"detail(get_by_id) 4 queries = {(time.perf_counter()-t0)*1000:.2f}ms")
