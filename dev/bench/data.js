window.BENCHMARK_DATA = {
  "lastUpdate": 1792990287043,
  "repoUrl": "https://github.com/Arete-Consortium/rebellion",
  "entries": {
    "Rust Benchmarks": [
      {
        "commit": {
          "author": {
            "name": "Arete-Consortium",
            "username": "Arete-Consortium"
          },
          "committer": {
            "name": "Arete-Consortium",
            "username": "Arete-Consortium"
          },
          "id": "fba7b46c3fa11c28f36144ce33e1f0389d42e27a",
          "message": "fix(ci): unblock security and benchmark gates",
          "timestamp": "2026-08-07T10:06:09Z",
          "url": "https://github.com/Arete-Consortium/rebellion/pull/58/commits/fba7b46c3fa11c28f36144ce33e1f0389d42e27a"
        },
        "date": 1792938575017,
        "tool": "cargo",
        "benches": [
          {
            "name": "score_on_kill_x1000",
            "value": 2147,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "spatial_grid_insert_query_500",
            "value": 21610,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "heat_classify_x10000",
            "value": 25106,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "combo_update_1000_frames",
            "value": 3230,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "score_get_grade_x10000",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AreteDriver@gmail.com",
            "name": "James C. Young",
            "username": "AreteDriver"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4a74af008e102e5382369fd2ee216b48e15fac8e",
          "message": "fix(ci): unblock security and benchmark gates (#58)\n\nResolve current RustSec advisories and make benchmark failures fail closed while preserving production release panic behavior.",
          "timestamp": "2026-10-25T21:15:22-07:00",
          "tree_id": "d12e96ef2a1867359d0c3f5d1e11fa6cdac31354",
          "url": "https://github.com/Arete-Consortium/rebellion/commit/4a74af008e102e5382369fd2ee216b48e15fac8e"
        },
        "date": 1792990282012,
        "tool": "cargo",
        "benches": [
          {
            "name": "score_on_kill_x1000",
            "value": 2831,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "spatial_grid_insert_query_500",
            "value": 28096,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "heat_classify_x10000",
            "value": 24559,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "combo_update_1000_frames",
            "value": 4326,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "score_get_grade_x10000",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}