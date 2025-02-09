# Domains

## Introduction


## Create a new Domain

cd to domains directory

1. run cargo command

   ```
   cargo domain new --name {domain_name}
   ```

2. choose the domain type

   ```
   1. Common
   2. Fs
   3. Driver
   ```
3. input the domain interface name

   ```
   {interface_name}
   ```

4. update domain-list.toml


## Build
```
cargo domain --help # Display help
cargo domain build-all -l "" # Build all domains
cargo domain build -n syscall -l "" # Build syscall domain
```
