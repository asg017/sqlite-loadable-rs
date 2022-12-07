#!/bin/bash

end=1e6
hyperfine --warmup 10 --export-json=results.json \
  "./base.sh $end" \
  "./series_c.sh $end" \
  "./series_rs.sh $end" \
  "./series_go.sh $end" 