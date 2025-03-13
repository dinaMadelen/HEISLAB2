# Elevator

This is a project for the course TTK4145 - Real-time Programming at NTNU.
The task is to implement a fault tolerant distributed system for controlling *n* amount of elevators.

## Development
If Elixir isn't installed, you can install it via `install_elixir.sh` if your OS supports apt.

Make sure to run `mix deps.get` if you want to run the project in `:dev`, this isn't needed if you only want to compile and run the project.
Docs can be generated with `mix docs`.

Always run `mix format` and test `mix test` before pushing.
Running `mix credo` is also recommended.

## How to run
1. Make sure Erlang/Elixir is installed.
2. Run `compile_linux.sh` or `compile_windows.bat` depending on your OS.
3. Run `start_elevator.*` inside the `releases/<compiled_folder>`

## Verifying core functions
1. Run `mix test` and make sure `MIX_ENV=test`.
