#!/usr/bin/env bash

set -e
 
/app/demo server &

for (( ; ; ))
do
  sleep 5s
  /app/demo client  
done
