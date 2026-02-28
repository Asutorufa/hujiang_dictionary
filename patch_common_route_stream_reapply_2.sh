#!/bin/bash
sed -i 's/#\[derive(Debug, Clone)\]//g' common/src/route.rs
