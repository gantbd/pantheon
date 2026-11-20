workspace = Path.expand("../../..", __DIR__)
rune_root = Path.join(workspace, "rune")
port_file = System.fetch_env!("PANTHEON_J1_UADD_PORT_FILE")
token_file = System.fetch_env!("PANTHEON_J1_UADD_TOKEN_FILE")

{:ok, registry} =
  RuneMcp.Server.start_link(
    namespace_map: %{
      "rune" => Path.join(rune_root, "sme"),
      "axon-sme" => Path.join(workspace, "axon/sme")
    },
    rune_loader_path: Path.join(rune_root, "bin/rune_loader"),
    ref_resolve_path: Path.join(rune_root, "bin/ref_resolve"),
    lattice_check_path: Path.join(rune_root, "bin/lattice_check"),
    smith_path: Path.join(rune_root, "bin/smith"),
    fold_check_path: Path.join(rune_root, "bin/fold_check")
  )

:ok =
  RuneMcp.Server.load(
    registry,
    "rune/sme/triggers",
    Path.join(rune_root, "sme/triggers.fold.rune")
  )

:ok =
  RuneMcp.Server.load(
    registry,
    "axon/sme/axon-concerns",
    Path.join(workspace, "axon/sme/axon-concerns.lattice.rune")
  )

:ok =
  RuneMcp.Server.load(
    registry,
    "pantheon/design/pantheon-concerns",
    Path.join(workspace, "pantheon/design/runes/pantheon-concerns.lattice.rune")
  )

secret = :crypto.strong_rand_bytes(32) |> Base.url_encode64(padding: false)
{:ok, auth} = Dendrite.Auth.start_link(clients: %{"pantheon-j1-uadd" => secret})

{:ok, %{access_token: token}} =
  Dendrite.Auth.issue_token(auth, "pantheon-j1-uadd", secret)

{:ok, http} =
  Dendrite.HTTP.start_link(
    registry: registry,
    auth: auth,
    auth_mode: :required,
    port: 0
  )

:ok = File.write(port_file, Integer.to_string(Dendrite.HTTP.port(http)), [:exclusive])
:ok = File.write(token_file, token, [:exclusive])
:ok = File.chmod(token_file, 0o600)
Process.sleep(:infinity)
