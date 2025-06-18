# /home/dweese/dev/rust/rabbitmq_workspace/bestinf.sh
# Project structure overview
tree -I 'target|node_modules|.git' -a

# Quick file sizes and mod times
find . -name "*.rs" -o -name "Cargo.toml" | xargs ls -la

# Key files content
cat Cargo.toml */Cargo.toml
head -20 */src/lib.rs
