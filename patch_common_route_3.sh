#!/bin/bash
sed -i 's/self.workers_ai\.m2m100_1_2b/self.translator.m2m100_1_2b/g' common/src/route.rs
sed -i 's/self.workers_ai\n                    .m2m100_1_2b/self.translator\n                    .m2m100_1_2b/g' common/src/route.rs
