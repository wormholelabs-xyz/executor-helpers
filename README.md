# Executor Helpers

This repo contains helpers that are useful for relaying messages supported by the [Executor framework](https://github.com/wormholelabs-xyz/example-messaging-executor). Particularly, these contracts focus on performing relays and drop-offs in a single transaction on chains that don't natively support client-side transaction composition.

- EVM
  - [CCTPv1ReceiveWithGasDropOff](./src/CCTPv1ReceiveWithGasDropOff.sol)
  - [CCTPv2ReceiveWithGasDropOff](./src/CCTPv2ReceiveWithGasDropOff.sol)
  - [MultiReceiveWithGasDropOff](./src/MultiReceiveWithGasDropOff.sol)
  - [VAAv1ReceiveWithGasDropOff](./src/VAAv1ReceiveWithGasDropOff.sol)
- Aptos
  - [cctp_v1_receive_with_gas_drop_off](./aptos/cctp_v1_receive_with_gas_drop_off/sources/cctp_v1_receive_with_gas_drop_off.move)

⚠ **This software is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
implied. See the License for the specific language governing permissions and limitations under the License.** Or plainly
spoken - this is a very complex piece of software which targets a bleeding-edge, experimental smart contract runtime.
Mistakes happen, and no matter how hard you try and whether you pay someone to audit it, it may eat your tokens, set
your printer on fire or startle your cat. Cryptocurrencies are a high-risk investment, no matter how fancy.
