defmodule OrderLightController.CoreTest do
  @moduledoc """
  Tests for OrderMap.Core functions
  """
  use ExUnit.Case

  setup do
    order1 = %Order{floor: 1, type: :cab}
    order2 = %Order{floor: 2, type: :hall_up}
    order3 = %Order{floor: 3, type: :hall_down}
    order4 = %Order{floor: 4, type: :cab}

    order_map = %{
      node1: [order1],
      node2: [order2, order3, order4],
      node3: []
    }

    {:ok, order1: order1, order2: order2, order3: order3, order4: order4, order_map: order_map}
  end

  describe "get_active_buttons/1" do
    test "Flattens the order_map with tags", %{
      order1: order1,
      order2: order2,
      order3: order3,
      order4: order4,
      order_map: order_map
    } do
      flattened_order_map = [
        {:node2, order2},
        {:node2, order3},
        {:node2, order4},
        {:node1, order1}
      ]

      assert OrderLightController.Core.get_active_buttons(order_map) == flattened_order_map
    end

    test "Returns empty list when order_map is empty" do
      order_map = %{}
      assert OrderLightController.Core.get_active_buttons(order_map) == []
    end
  end

  describe "tag_orders/2" do
    test "Returns a list with every element tagged", %{order1: order1, order2: order2} do
      list_to_tag = [order1, order2]
      node_name = :node1
      expected_tagged_list = [{node_name, order1}, {node_name, order2}]

      assert OrderLightController.Core.tag_orders(list_to_tag, node_name) == expected_tagged_list
    end

    test "Returns empty list when list_to_tag is empty" do
      list_to_tag = []
      node_name = :node1
      assert OrderLightController.Core.tag_orders(list_to_tag, node_name) == []
    end
  end
end
