#!/bin/bash
git restore common/src/route.rs
git restore common/src/tg.rs

sed -i 's/T2: WorkersAI/T2: crate::ai::Translator/g' common/src/route.rs
sed -i 's/T2: WorkersAI/T2: crate::ai::Translator/g' common/src/tg.rs
sed -i 's/self.workers_ai.enabled()/self.workers_ai.is_some()/g' common/src/route.rs
sed -i 's/self.workers_ai.models()/self.workers_ai.as_ref().unwrap().models()/g' common/src/route.rs
sed -i 's/self.workers_ai\.m2m100_1_2b/self.translator.m2m100_1_2b/g' common/src/route.rs
sed -i 's/opt.workers_ai.m2m100_1_2b/opt.translator.m2m100_1_2b/g' common/src/tg.rs
