defmodule ButtonPoller.CoreTest do
  @moduledoc """
  Tests for ButtonPoller.Core functions
  """
  use ExUnit.Case

  @top_floor 3
  @bottom_floor 0

  describe "get_buttons_of_type/3" do
    test "Returns all possible orders of a single button type, :hall_up" do
      hall_up = [
        %Order{floor: 0, type: :hall_up},
        %Order{floor: 1, type: :hall_up}
      ]

      assert ButtonPoller.Core.get_buttons_of_type(:hall_up, @bottom_floor, @top_floor - 1) ==
               hall_up
    end

    test "Returns all possible orders of a single button type, :hall_down" do
      hall_down = [
        %Order{floor: 2, type: :hall_down},
        %Order{floor: 3, type: :hall_down}
      ]

      assert ButtonPoller.Core.get_buttons_of_type(:hall_down, @bottom_floor + 1, @top_floor) ==
               hall_down
    end

    test "Returns all possible orders of a single button type, :cab" do
      cab = [
        %Order{floor: -1, type: :cab},
        %Order{floor: 0, type: :cab},
        %Order{floor: 1, type: :cab},
        %Order{floor: 2, type: :cab},
        %Order{floor: 3, type: :cab},
        %Order{floor: 4, type: :cab}
      ]

      assert ButtonPoller.Core.get_buttons_of_type(:cab, @bottom_floor - 1, @top_floor + 1) == cab
    end
  end

  describe "get_all_buttons/2" do
    test "Returns the correct map of all possible buttons, standard floors" do
      standard_map = [
        %Order{floor: 0, type: :hall_up},
        %Order{floor: 1, type: :hall_up},
        %Order{floor: 2, type: :hall_up},
        %Order{floor: 1, type: :hall_down},
        %Order{floor: 2, type: :hall_down},
        %Order{floor: 3, type: :hall_down},
        %Order{floor: 0, type: :cab},
        %Order{floor: 1, type: :cab},
        %Order{floor: 2, type: :cab},
        %Order{floor: 3, type: :cab}
      ]

      assert ButtonPoller.Core.get_all_buttons(@bottom_floor, @top_floor) == standard_map
    end

    test "Returns the correct map of all possible buttons, non-standard floors" do
      standard_map = [
        %Order{floor: -2, type: :hall_up},
        %Order{floor: -1, type: :hall_up},
        %Order{floor: 0, type: :hall_up},
        %Order{floor: 1, type: :hall_up},
        %Order{floor: -1, type: :hall_down},
        %Order{floor: 0, type: :hall_down},
        %Order{floor: 1, type: :hall_down},
        %Order{floor: 2, type: :hall_down},
        %Order{floor: -2, type: :cab},
        %Order{floor: -1, type: :cab},
        %Order{floor: 0, type: :cab},
        %Order{floor: 1, type: :cab},
        %Order{floor: 2, type: :cab}
      ]

      assert ButtonPoller.Core.get_all_buttons(-2, 2) == standard_map
    end
  end
end
