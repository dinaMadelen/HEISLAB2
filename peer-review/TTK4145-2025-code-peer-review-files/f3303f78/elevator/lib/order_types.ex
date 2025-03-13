defmodule Order do
  @moduledoc """
  Container for the order struct definition.
  """
  # Types and structs  ----------------------------------------------------------------------------
  @enforce_keys [:floor, :type]
  defstruct [:floor, :type]

  @typedoc """
  This type is used as type definition for the order struct.
  """
  @type t :: %Order{floor: integer, type: atom}
end

defmodule OrderMap do
  @moduledoc """
  Container for the order_map type definition.
  """
  # Types and structs  ----------------------------------------------------------------------------
  @type order :: Order.t()
  @typedoc """
  This type is used as type definition for the map containing every order across all nodes.
  """
  @type t :: %{node => [order]}
end
