defmodule ElevatorFSMCoreTest do
  @moduledoc """
  Tests for ElevatorFSM.Core
  """
  use ExUnit.Case

  describe "get_new_direction/2" do
    test "returns :up when destination floor is higher than current floor" do
      assert ElevatorFSM.Core.get_new_direction(1, 3) == :up
    end

    test "returns :down when destination floor is lower than current floor" do
      assert ElevatorFSM.Core.get_new_direction(3, -1) == :down
    end

    test "returns :at_floor when destination floor is the same as current floor" do
      assert ElevatorFSM.Core.get_new_direction(2, 2) == :at_floor
    end
  end
end
