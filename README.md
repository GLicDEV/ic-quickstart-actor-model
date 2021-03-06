# ic_quickstart_actor_model

Welcome to your new ic_quickstart_actor_model project and to the internet computer development community. By default, creating a new project adds this README and some template files to your project directory. You can edit these template files to customize your project and to include your own code to speed up the development cycle.

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with ic_quickstart_actor_model, see the following documentation available online:

- [Quick Start](https://smartcontracts.org/docs/quickstart/quickstart-intro.html)
- [SDK Developer Tools](https://smartcontracts.org/docs/developers-guide/sdk-guide.html)
- [Rust Canister Devlopment Guide](https://smartcontracts.org/docs/rust-guide/rust-intro.html)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://smartcontracts.org/docs/candid-guide/candid-intro.html)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.ic0.app)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd ic_quickstart_actor_model/
dfx help
dfx config --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

After the canister is deployed, we need to load the canister's WASM binary so that the canister can spawn new canisters. Depending on your dfx settings and previous projects ran, you might need to alter a canister_id in this step.

```
# go to the wasm_loader folder
cd src/wasm_loader

chmod +x post_deploy.sh

cat post_deploy.sh

# check that the 2'nd parameter, the canister_id matches the canister_id from the output of "dfx deploy" in a previous step. If they don't match, edit this file, and continue

./post_deploy.sh
```

If you get a ```response: true``` the wasm was installed correctly.

Once the job completes, your application will be available at `http://localhost:8000?canisterId={asset_canister_id}`.

### Possible errors

> sh: 1: webpack: not found

```bash
npm install --save-dev webpack

```
