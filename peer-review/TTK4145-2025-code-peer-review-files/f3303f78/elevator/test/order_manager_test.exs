defmodule OrderManager.CoreTest do
  @moduledoc """
  Tests for OrderManager.Core functions
  """
  use ExUnit.Case

  describe "get_next_order/4" do
    test "test for in-path order in opposite down direction" do
      order_map = %{
        node1: [
          %Order{floor: 3, type: :hall_down},
          %Order{floor: 2, type: :hall_down},
          %Order{floor: 1, type: :hall_up},
          %Order{floor: 0, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 1, :down) == %Order{
               floor: 0,
               type: :hall_up
             }
    end

    test "test for in-path order in opposite up direction" do
      order_map = %{
        node1: [
          %Order{floor: 3, type: :hall_down},
          %Order{floor: 2, type: :hall_down},
          %Order{floor: 1, type: :hall_up},
          %Order{floor: 0, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 2, :up) == %Order{
               floor: 3,
               type: :hall_down
             }
    end

    test "test for not-in-path order in opposite up direction" do
      order_map = %{
        node1: [
          %Order{floor: 3, type: :hall_down},
          %Order{floor: 2, type: :hall_up},
          %Order{floor: 1, type: :hall_down},
          %Order{floor: 0, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 4, :up) == %Order{
               floor: 3,
               type: :hall_down
             }
    end

    test "test for not-in-path order in opposite down direction" do
      order_map = %{
        node1: [
          %Order{floor: 3, type: :hall_down},
          %Order{floor: 2, type: :hall_up},
          %Order{floor: 1, type: :hall_down},
          %Order{floor: 0, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, -2, :down) == %Order{
               floor: 0,
               type: :hall_up
             }
    end

    test "test for in-path order in same up direction" do
      order_map = %{
        node1: [
          %Order{floor: 3, type: :hall_down},
          %Order{floor: 2, type: :hall_up},
          %Order{floor: 1, type: :hall_down},
          %Order{floor: 0, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 1, :up) == %Order{
               floor: 2,
               type: :hall_up
             }
    end

    test "test for in-path order cab call" do
      order_map = %{
        node1: [
          %Order{floor: 3, type: :hall_up},
          %Order{floor: 2, type: :cab},
          %Order{floor: 1, type: :hall_down},
          %Order{floor: 0, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 1, :up) == %Order{
               floor: 2,
               type: :cab
             }
    end

    test "test for in-path order in same down direction" do
      order_map = %{
        node1: [
          %Order{floor: 3, type: :hall_down},
          %Order{floor: 2, type: :hall_up},
          %Order{floor: 1, type: :hall_down},
          %Order{floor: 0, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 8, :down) == %Order{
               floor: 3,
               type: :hall_down
             }
    end

    test "test for not-in-path order in same down direction" do
      order_map = %{
        node1: [
          %Order{floor: 3, type: :hall_down},
          %Order{floor: 2, type: :hall_down}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 1, :down) == %Order{
               floor: 3,
               type: :hall_down
             }
    end

    test "test for not-in-path order cab call" do
      order_map = %{
        node1: [
          %Order{floor: 4, type: :hall_up},
          %Order{floor: 4, type: :hall_down},
          %Order{floor: 3, type: :cab},
          %Order{floor: 2, type: :hall_down}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 1, :down) == %Order{
               floor: 3,
               type: :cab
             }
    end

    test "test for not-in-path order in same up direction" do
      order_map = %{
        node1: [
          %Order{floor: 3, type: :hall_up},
          %Order{floor: 2, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 7, :up) == %Order{
               floor: 2,
               type: :hall_up
             }
    end

    test "test for not-in-path order in opposite direction with same up direction order on same floor" do
      order_map = %{
        node1: [
          %Order{floor: 2, type: :hall_down},
          %Order{floor: 2, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 3, :up) == %Order{
               floor: 2,
               type: :hall_down
             }
    end

    test "test for not-in-path order in opposite direction with same down direction order on same floor" do
      order_map = %{
        node1: [
          %Order{floor: 2, type: :hall_down},
          %Order{floor: 2, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 1, :down) == %Order{
               floor: 2,
               type: :hall_up
             }
    end

    test "test for in-path order in opposite direction with same up direction order on same floor" do
      order_map = %{
        node1: [
          %Order{floor: 2, type: :hall_down},
          %Order{floor: 2, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 2, :up) == %Order{
               floor: 2,
               type: :hall_up
             }
    end

    test "test for in-path order in opposite direction with same down direction order on same floor" do
      order_map = %{
        node1: [
          %Order{floor: 2, type: :hall_down},
          %Order{floor: 2, type: :hall_up}
        ]
      }

      assert OrderManager.Core.get_next_order(order_map, :node1, 2, :down) == %Order{
               floor: 2,
               type: :hall_down
             }
    end

    test "test for no more orders" do
      order_map = %{node1: []}

      assert OrderManager.Core.get_next_order(order_map, :node1, 2, :down) == :no_orders
    end
  end

  describe "optimal_destination/2" do
    test "test for up direction orders" do
      order_list = [
        %Order{floor: 3, type: :hall_down},
        %Order{floor: 2, type: :hall_up},
        %Order{floor: 1, type: :hall_down}
      ]

      assert OrderManager.Core.optimal_destination(order_list, :hall_up) == %Order{
               floor: 2,
               type: :hall_up
             }
    end

    test "test for down direction orders" do
      order_list = [
        %Order{floor: 4, type: :hall_up},
        %Order{floor: 3, type: :hall_down},
        %Order{floor: 1, type: :hall_down}
      ]

      assert OrderManager.Core.optimal_destination(order_list, :hall_down) == %Order{
               floor: 3,
               type: :hall_down
             }
    end

    test "test for no same up direction orders" do
      order_list = [
        %Order{floor: 9, type: :hall_down},
        %Order{floor: 1, type: :hall_down}
      ]

      assert OrderManager.Core.optimal_destination(order_list, :hall_up) == %Order{
               floor: 9,
               type: :hall_down
             }
    end

    test "test for no down direction orders" do
      order_list = [
        %Order{floor: 3, type: :hall_up},
        %Order{floor: -5, type: :hall_up}
      ]

      assert OrderManager.Core.optimal_destination(order_list, :hall_down) == %Order{
               floor: -5,
               type: :hall_up
             }
    end

    test "test for cab orders" do
      order_list = [
        %Order{floor: 3, type: :cab},
        %Order{floor: 2, type: :hall_down},
        %Order{floor: 1, type: :hall_down}
      ]

      assert OrderManager.Core.optimal_destination(order_list, :hall_up) == %Order{
               floor: 3,
               type: :cab
             }
    end

    test "test for cab order in middle" do
      order_list = [
        %Order{floor: 3, type: :hall_down},
        %Order{floor: 2, type: :cab},
        %Order{floor: 1, type: :hall_down}
      ]

      assert OrderManager.Core.optimal_destination(order_list, :hall_down) == %Order{
               floor: 3,
               type: :hall_down
             }
    end

    test "test for cab order with same direction order above" do
      order_list = [
        %Order{floor: 4, type: :hall_up},
        %Order{floor: 3, type: :cab},
        %Order{floor: 2, type: :hall_down},
        %Order{floor: 1, type: :hall_down}
      ]

      assert OrderManager.Core.optimal_destination(order_list, :hall_up) == %Order{
               floor: 3,
               type: :cab
             }
    end
  end
end
