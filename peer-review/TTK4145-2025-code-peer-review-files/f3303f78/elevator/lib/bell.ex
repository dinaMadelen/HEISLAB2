defmodule Bell do
  @moduledoc """
  This module contains the function for bell notification of floor arrival.
  """

  @doc """
  This function plays a bell sound.
  """
  def play() do
    file_path = "bell.wav"
    Task.start(fn -> System.cmd("ffplay", ["-nodisp", file_path], stderr_to_stdout: true) end)
  end
end
