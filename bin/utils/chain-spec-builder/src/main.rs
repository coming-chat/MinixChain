// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::{fs, path::{Path, PathBuf}, sync::Arc};

use ansi_term::Style;
use rand::{Rng, distributions::Alphanumeric, rngs::OsRng};
use structopt::StructOpt;

use sc_keystore::LocalKeystore;
use minix::chain_spec::{self, AccountId};
use sp_core::{
	crypto::{Public, Ss58Codec},
};
use sp_keystore::{SyncCryptoStorePtr, SyncCryptoStore};
use sc_service::{Properties};

/// A utility to easily create a testnet chain spec definition with a given set
/// of authorities and endowed accounts and/or generate random accounts.
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum ChainSpecBuilder {
	/// Create a new chain spec with the given authorities, endowed and sudo
	/// accounts.
	New {
		/// Authority key seed.
		#[structopt(long, short, required = true)]
		authority_seeds: Vec<String>,
		/// Endowed account address (SS58 format).
		#[structopt(long, short)]
		endowed_accounts: Vec<String>,
		/// Sudo account address (SS58 format).
		#[structopt(long, short)]
		sudo_account: String,
		/// The path where the chain spec should be saved.
		#[structopt(long, short, default_value = "./chain_spec.json")]
		chain_spec_path: PathBuf,
	},
	/// Create a new chain spec with the given number of authorities and endowed
	/// accounts. Random keys will be generated as required.
	Generate {
		/// The number of authorities.
		#[structopt(long, short)]
		authorities: usize,
		/// Endowed account address (SS58 format).
		#[structopt(long, short)]
		endowed_accounts: Vec<String>,
		/// Sudo account address (SS58 format).
		#[structopt(long, short)]
		sudo_account: String,
		/// The path where the chain spec should be saved.
		#[structopt(long, short, default_value = "./chain_spec.json")]
		chain_spec_path: PathBuf,
		/// Path to use when saving generated keystores for each authority.
		///
		/// At this path, a new folder will be created for each authority's
		/// keystore named `auth-$i` where `i` is the authority index, i.e.
		/// `auth-0`, `auth-1`, etc.
		#[structopt(long, short)]
		keystore_path: Option<PathBuf>,
	},
}

impl ChainSpecBuilder {
	/// Returns the path where the chain spec should be saved.
	fn chain_spec_path(&self) -> &Path {
		match self {
			ChainSpecBuilder::New { chain_spec_path, .. } =>
				chain_spec_path.as_path(),
			ChainSpecBuilder::Generate { chain_spec_path, .. } =>
				chain_spec_path.as_path(),
		}
	}
}

fn genesis_constructor(
	authority_seeds: &[String],
	endowed_accounts: &[AccountId],
	sudo_account: &AccountId,
) -> chain_spec::GenesisConfig {
	let authorities = authority_seeds
		.iter()
		.map(AsRef::as_ref)
		.map(chain_spec::authority_keys_from_seed)
		.collect::<Vec<_>>();

	let enable_println = true;

	chain_spec::minix_genesis(
		authorities,
		sudo_account.clone(),
		endowed_accounts.to_vec(),
		enable_println,
	)
}

fn generate_chain_spec(
	authority_seeds: Vec<String>,
	endowed_accounts: Vec<String>,
	sudo_account: String,
) -> Result<String, String> {
	let parse_account = |address: String| {
		AccountId::from_string(&address)
			.map_err(|err| format!("Failed to parse account address: {:?}", err))
	};

	let endowed_accounts =
		endowed_accounts.into_iter().map(parse_account).collect::<Result<Vec<_>, String>>()?;

	let sudo_account = parse_account(sudo_account)?;

	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "mini".into());
	properties.insert("tokenDecimals".into(), 8.into());

	let chain_spec = chain_spec::ChainSpec::from_genesis(
		"MiniX",
		"MiniX",
		sc_chain_spec::ChainType::Live,
		move || genesis_constructor(&authority_seeds, &endowed_accounts, &sudo_account),
		vec![],
		None,
		None,
		Some(properties),
		Default::default(),
	);

	chain_spec.as_json(false).map_err(|err| err)
}

fn generate_authority_keys_and_store(
	seeds: &[String],
	keystore_path: &Path,
) -> Result<(), String> {
	for (n, seed) in seeds.into_iter().enumerate() {
		let keystore: SyncCryptoStorePtr = Arc::new(LocalKeystore::open(
			keystore_path.join(format!("auth-{}", n)),
			None,
		).map_err(|err| err.to_string())?);

		let (aura, grandpa) =
			chain_spec::authority_keys_from_seed(seed);

        let insert_key = |key_type, public| {
            SyncCryptoStore::insert_unknown(
                &*keystore,
                key_type,
                &format!("//{}", seed),
                public,
            ).map_err(|_| format!("Failed to insert key: {}", grandpa))
        };

		insert_key(
			sp_core::crypto::key_types::AURA,
			aura.as_slice(),
		)?;

		insert_key(
			sp_core::crypto::key_types::GRANDPA,
			grandpa.as_slice(),
		)?;
	}

	Ok(())
}

fn print_seeds(
	authority_seeds: &[String],
) {
	let header = Style::new().bold().underline();
	let entry = Style::new().bold();

	println!("{}", header.paint("Authority seeds"));

	for (n, seed) in authority_seeds.iter().enumerate() {
		println!("{} //{}",
			entry.paint(format!("auth-{}:", n)),
			seed,
		);
	}

	println!();
}

fn main() -> Result<(), String> {
	#[cfg(build_type = "debug")]
	println!(
		"The chain spec builder builds a chain specification that includes a Substrate runtime \
		 compiled as WASM. To ensure proper functioning of the included runtime compile (or run) \
		 the chain spec builder binary in `--release` mode.\n",
	);

	let builder = ChainSpecBuilder::from_args();
	let chain_spec_path = builder.chain_spec_path().to_path_buf();

	let (authority_seeds, endowed_accounts, sudo_account) = match builder {
		ChainSpecBuilder::Generate { authorities, endowed_accounts, keystore_path, sudo_account, .. } => {
			let authorities = authorities.max(1);
			let rand_str = || -> String { OsRng.sample_iter(&Alphanumeric).take(32).collect() };

			let authority_seeds = (0..authorities).map(|_| rand_str()).collect::<Vec<_>>();

			print_seeds(&authority_seeds/*, &endowed_seeds, &sudo_seed*/);

			if let Some(keystore_path) = keystore_path {
				generate_authority_keys_and_store(
					&authority_seeds,
					&keystore_path,
				)?;
			}


			(authority_seeds, endowed_accounts, sudo_account)
		}
		ChainSpecBuilder::New {
			authority_seeds,
			endowed_accounts,
			sudo_account,
			..
		} => (authority_seeds, endowed_accounts, sudo_account),
	};

	let json = generate_chain_spec(
		authority_seeds,
		endowed_accounts,
		sudo_account,
	)?;

	fs::write(chain_spec_path, json).map_err(|err| err.to_string())
}
