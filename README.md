# Executor Helpers

This repo contains helpers that are useful for relaying messages supported by the [Executor framework](https://github.com/wormholelabs-xyz/example-messaging-executor). Particularly, these contracts focus on performing relays and drop-offs in a single transaction on chains that don't natively support client-side transaction composition.

- EVM
  - [CCTPv1ReceiveWithGasDropOff](./src/CCTPv1ReceiveWithGasDropOff.sol)
  - [CCTPv2ReceiveWithGasDropOff](./src/CCTPv2ReceiveWithGasDropOff.sol)
  - [MultiReceiveWithGasDropOff](./src/MultiReceiveWithGasDropOff.sol)
  - [VAAv1ReceiveWithGasDropOff](./src/VAAv1ReceiveWithGasDropOff.sol)
- Aptos
  - [cctp_v1_receive_with_gas_drop_off](./aptos/cctp_v1_receive_with_gas_drop_off/sources/cctp_v1_receive_with_gas_drop_off.move)
- SVM (Solana)
  - [relay_cost_protection](./svm/relay_cost_protection/programs/relay_cost_protection/src/lib.rs)

## Development container

Open this repository in VS Code and select "Dev Containers: Reopen in Container".
The container has Foundry, Rust, Solana CLI, Anchor, Aptos CLI, Bun, surfpool, just, and Claude Code.
[.devcontainer/devcontainer.json](./.devcontainer/devcontainer.json) pins each tool to one version and one SHA-256.

The [castellan](https://github.com/wormholelabs-xyz/castellan) firewall permits network traffic only to the hosts in [.devcontainer/allowed-domains.txt](./.devcontainer/allowed-domains.txt).
To add a host, add it to that file and rebuild the container.

To check the pinned tool versions and build each runtime, run:

```bash
bash .devcontainer/verify-toolchain.sh
```

On Apple Silicon, the first build compiles the Solana CLI from source.
This step takes about 10 minutes on a 16-core machine.
Later builds use the Docker cache.

⚠ **This software is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
implied. See the License for the specific language governing permissions and limitations under the License.** Or plainly
spoken - this is a very complex piece of software which targets a bleeding-edge, experimental smart contract runtime.
Mistakes happen, and no matter how hard you try and whether you pay someone to audit it, it may eat your tokens, set
your printer on fire or startle your cat. Cryptocurrencies are a high-risk investment, no matter how fancy.
