defmodule NetworkTest do
  @moduledoc """
  Tests for the Network.Core functions.
  """
  use ExUnit.Case

  describe "ip_to_string/1" do
    test "converts a valid IPv4 tuple to string" do
      assert Network.Core.ip_to_string({192, 168, 1, 1}) == "192.168.1.1"
    end

    test "converts a valid IPv6 tuple (int) to string" do
      assert Network.Core.ip_to_string({65535, 0, 26235, 43126, 65535, 0, 26235, 43126}) ==
               "ffff:0:667b:a876:ffff:0:667b:a876"
    end

    test "converts a valid IPv6 tuple (hex) to string" do
      assert Network.Core.ip_to_string({0xFFFF, 0x0, 0x667B, 0xA876, 0xFFFF, 0x0, 0x667B, 0xA876}) ==
               "ffff:0:667b:a876:ffff:0:667b:a876"
    end

    test "raises Protocol.UndefinedError if the argument is not a tuple" do
      assert_raise Protocol.UndefinedError, fn ->
        Network.Core.ip_to_string(12)
      end
    end

    test "raises Protocol.UndefinedError if the tuple doesn't match IPv4 or IPv6" do
      assert_raise Protocol.UndefinedError, fn ->
        Network.Core.ip_to_string({192, 168, 1, 1, 1})
      end
    end

    test "raises Protocol.UndefinedError if the tuple has less than 4 elements" do
      assert_raise Protocol.UndefinedError, fn ->
        Network.Core.ip_to_string({192, 168, 1})
      end
    end

    test "raises Protocol.UndefinedError if IPv4 address got too high number" do
      assert_raise Protocol.UndefinedError, fn ->
        Network.Core.ip_to_string({192, 168, 1, 256})
      end
    end

    test "raises Protocol.UndefinedError if IPv6 address got too high number" do
      assert_raise Protocol.UndefinedError, fn ->
        Network.Core.ip_to_string({65536, 0, 26235, 43126, 65535, 0, 26235, 43126})
      end
    end
  end

  describe "node_connected?/2" do
    test "returns true if node name + node domain is in the list" do
      assert Network.Core.node_connected?("node1@127.0.0.1", [
               :"node1@127.0.0.1",
               :"node2@127.0.0.1"
             ])
    end

    test "returns false if node domain is mismatched in the list" do
      refute Network.Core.node_connected?("node1@127.0.0.1", [
               :"node1@127.0.0.2",
               :"node2@127.0.0.1"
             ])
    end

    test "returns false if node name is mismatched in the list" do
      refute Network.Core.node_connected?("g77@127.0.0.1", [:"node1@127.0.0.2", :"g78@127.0.0.1"])
    end

    test "returns false if the list is empty" do
      refute Network.Core.node_connected?("node1@127.0.0.1", [])
    end

    test "returns true if short node name is in the list" do
      assert Network.Core.node_connected?("node1@localhost", [:node1@localhost, :node2@localhost])
    end

    test "returns false if short node domain is mismatched in the list" do
      refute Network.Core.node_connected?("node1@localhost", [
               :"node1@my-computer",
               :node2@localhost
             ])
    end

    test "returns false if short node name is mismatched in the list" do
      refute Network.Core.node_connected?("g77@localhost", [:"node1@my-computer", :g78@localhost])
    end

    test "returns false if the list is empty for short node names" do
      refute Network.Core.node_connected?("node1@localhost", [])
    end

    test "returns ArgumentError if second argument isn't a list of atoms" do
      assert_raise ArgumentError, fn ->
        Network.Core.node_connected?("node1@255.255.255.255", ["node1@255.255.255.255"])
      end
    end

    test "returns false if first arg is an atom and matches and atom in the list" do
      refute Network.Core.node_connected?(:"node1@255.255.255.255", [:"node1@255.255.255.255"])
    end
  end
end
