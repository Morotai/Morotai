# apcert_util
This utility provides semi-automation for new cluster certificate generation as described on [this link.](https://msazure.visualstudio.com/One/_git/Azure-Documents-Common?path=%2FProducts%2FAutopilot%2FAutopilot%2FAPPKI%2FAdministration.md&version=GBmaster&line=1&_a=preview&anchor=pilotfish-docs-contribution-guide)

**IMPORTANT:** This utility expects the following:
- It must be run from a **PFGold** enlistment command **ONLY**.
-  **%inetroot%** environment variable must be set in your local machine.

## How to Use
The apcert_util takes the following parameters:
1. Cluster name - This is a **required** parameter
2. Local working directory - This is an **optional** parameter. If no working directory is provided, a folder is created in the temp directory like this: **%TEMP%\mmmm-dd-yyyy**.
## Sample commands
- Single cluster: **apcert_util cluster01p c:\working_dir**
- Multiple clusters: **apcert_util cluster01p,cluster02p,cluster03p c:\working_dir**

#Creating a new cluster certificate requires the following 8 stages:
- **Stage-1** - Create all required artifacts (folders, files, etc.)

**NOTE:** apcert_util creates and updates all required artifacts. After running one of the commands above, continue with Stage-2.

- **Stage-2** - Sign and submit all artifacts to Source Depot
- **Stage-3** - Generate new certificate key-pair with PkiCmd 

**NOTE:** The required PkiCmd can be found inside the JSON file for each cluster located in the **Working_Dir** folder specified. i.e: **working_dir\cluster_name.json**.

- **Stage-4** - Sign and submit encrypted private key (APCA-CLUSTER_NAME_0.key.encr) from stage 3 to Source Depot
- **Stage-5** - Submit public key (Cluster_Name.cer) to PRSS for signing
- **Stage-6** - Sign and submit new PRSS signed certificate(s) to Source Depot

**NOTE** It can take PRSS up to 10 days to sign a new cluster certificate.

- **Stage-7** - Activate PRSS signed certificate(s) in PFGold by updating the local config.ini file
- **Stage-8** - Finally, sign and submit the updated config.ini file to Source Depot to complete the new cluster certificate process

## Setting up your dev box
The following tools are required to setup your Windows dev box **Rust** development:
1. Install [Rust 64bit](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe)
2. Install [Visual Studio C++ Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
3. Install [Visual Studio Code](https://code.visualstudio.com/Download#)
4. Install [Rust-Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer)

## For WSL on Windows
- Run this curl command: **curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh**

Information on other installs [can be found here](https://forge.rust-lang.org/infra/other-installation-methods.html)

## Running the application in your dev box
1. Clone the Azure-Compute-Move repo to your local dev box.
2. Navigate to the **apcert_util** folder and type **code .**.
3. Make desire changes on the desired Rust file(s) **(.rs)**.
4. From a command prompt, navigate to the root folder of apcert_util on the local copy of the repo and type the following commands: 
**cargo check** - If this check passes, then the code will compile
**cargo build --release** - This command will compile a production uptimized exe file in **.\target\release**
5. Follow the steps above to test the updated version of the utility.

## TIP: Run cargo fmt to format your Rust code.