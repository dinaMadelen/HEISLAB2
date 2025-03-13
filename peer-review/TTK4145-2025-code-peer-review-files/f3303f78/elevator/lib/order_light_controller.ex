defmodule OrderLightController do
  @moduledoc """
  This module is responsible for handling the management of order lights.
  """
  alias OrderLightController.Core, as: Core

  # Types and structs  ----------------------------------------------------------------------------
  @type from :: {pid, any}
  @type order :: Order.t()
  @type order_map :: OrderMap.t()

  # Config constants  -----------------------------------------------------------------------------
  @top_floor Application.compile_env!(:elevator, :top_floor)
  @bottom_floor Application.compile_env!(:elevator, :bottom_floor)

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Sets a button light on or off depending on status. Ignores call if the order is :cab and the
  node isn't Node.self()
  """
  @spec set_light(order, node, atom) :: :ok
  def set_light(order, node, status) do
    cond do
      order.type == :cab and node == Node.self() ->
        Driver.set_order_button_light(order.type, order.floor, status)

      order.type != :cab ->
        Driver.set_order_button_light(order.type, order.floor, status)

      true ->
        :ok
    end
  end

  @doc """
  Resets every light to :off, and every light in orders to :on.
  """
  @spec update_lights(order_map) :: :ok
  def update_lights(orders) do
    active_buttons = Core.get_active_buttons(orders)
    active_buttons |> Enum.each(fn {node, order} -> set_light(order, node, :on) end)

    ButtonPoller.Core.get_all_buttons(@bottom_floor, @top_floor)
    |> Enum.reject(fn button -> Enum.member?(active_buttons, {Node.self(), button}) end)
    |> Enum.each(fn button -> set_light(button, Node.self(), :off) end)
  end
end

defmodule OrderLightController.Core do
  @moduledoc """
  This module contains the core functions for the LightManager module, stateless.
  """
  # Types and structs  ----------------------------------------------------------------------------
  @type order :: Order.t()
  @type order_map :: OrderMap.t()

  # API functions  --------------------------------------------------------------------------------
  @doc """
  Returns a list of all buttons that should be active, and which node that is responsible for each
  button.
  """
  @spec get_active_buttons(order_map) :: [{atom, order}]
  def get_active_buttons(orders) do
    Map.keys(orders)
    |> Enum.reduce([], fn node, accumulator -> [tag_orders(orders[node], node) | accumulator] end)
    |> List.flatten()
  end

  @doc """
  Tags each order with the node that is responsible for it.
  """
  @spec tag_orders(order_map, atom) :: [{atom, order}]
  def tag_orders(order_list, node) do
    order_list |> Enum.map(fn order -> {node, order} end)
  end
end
