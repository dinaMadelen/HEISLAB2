defmodule Mix.Tasks.Debug do
  use Mix.Task

  @impl Mix.Task
  def run(args) do
    case args do
      [] ->
        IO.puts("enter node name: e.g node@127.0.0.1"); Kernel.exit(:normal) 
      _ ->
        Node.start(:"debug@127.0.0.1"); Node.set_cookie(:cool)
        Node.connect(String.to_atom(Enum.at(args, 0)))
    end
    loop()
  end
  
  def loop do
    IO.inspect(GenServer.call({Controller, :"node@127.0.0.1"}, :debug_state))

    Process.sleep(200)
    loop()
  end
end
