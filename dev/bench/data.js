window.BENCHMARK_DATA = {
  "lastUpdate": 1792938579934,
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
      }
    ]
  }
}