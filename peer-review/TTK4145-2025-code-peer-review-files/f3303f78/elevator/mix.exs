defmodule Elevator.MixProject do
  use Mix.Project

  def project do
    [
      app: :elevator,
      version: "0.8.0",
      elixir: "~> 1.13",
      start_permanent: Mix.env() == :prod,
      env: Mix.env(),
      deps: deps(),
      docs: docs(),
      releases: releases()
    ]
  end

  def application() do
    [
      extra_applications: [:logger, :runtime_tools],
      mod: {Main, []}
      # Switch with Main to simulate single elevator
      # mod: {ElevatorSimul, []}
    ]
  end

  defp releases() do
    [
      win_debug: [
        include_executables_for: [:windows],
        applications: [runtime_tools: :permanent],
        path: "releases/win_debug/"
      ],
      linux_debug: [
        include_executables_for: [:unix],
        applications: [runtime_tools: :permanent],
        path: "releases/linux_debug/"
      ]
    ]
  end

  defp deps() do
    [
      {:ex_doc, "~> 0.29", only: :dev, runtime: false},
      {:credo, "~> 1.7", only: :dev, runtime: false}
    ]
  end

  defp docs() do
    [
      main: "readme",
      extras: ["README.md"],
      assets: %{"exdoc_assets/" => "dist/"},
      before_closing_head_tag: &add_custom_css/1
    ]
  end

  def add_custom_css(:html) do
    ~S(    <link rel="stylesheet" href="dist/ex_doc.css" />)
  end

  def add_custom_css(:epub) do
  end
end
