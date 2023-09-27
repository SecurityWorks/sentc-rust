use sentc_crypto::crypto::{
	decrypt_raw_symmetric,
	decrypt_raw_symmetric_with_aad,
	decrypt_string_symmetric,
	decrypt_string_symmetric_with_aad,
	done_fetch_sym_key,
	encrypt_raw_symmetric,
	encrypt_raw_symmetric_with_aad,
	encrypt_string_symmetric,
	encrypt_string_symmetric_with_aad,
	encrypt_symmetric,
	encrypt_symmetric_with_aad,
	split_head_and_encrypted_data,
	split_head_and_encrypted_string,
};
use sentc_crypto::entities::keys::SymKeyFormatInt;
use sentc_crypto::sdk_common::crypto::EncryptedHead;

use crate::cache::l_one::L1Cache;
use crate::error::SentcError;
use crate::group::Group;
use crate::group_key;
use crate::net_helper::get_verify_key_internally_for_decrypt;

macro_rules! opt_sign {
	($self:expr, $c:expr, $sign:expr, |$sign_key:ident| $scope:block) => {
		if $sign {
			let user = $c
				.get_user(&$self.used_user_id)
				.await
				.ok_or(SentcError::UserNotFound)?;

			let user = user.read().await;

			let $sign_key = Some(user.get_newest_sign_key().ok_or(SentcError::KeyNotFound)?);
			$scope
		} else {
			let $sign_key = None;
			$scope
		}
	};
}

impl Group
{
	//raw encrypt

	pub async fn encrypt_raw(&self, data: &[u8], sign: bool, c: &L1Cache) -> Result<(EncryptedHead, Vec<u8>), SentcError>
	{
		opt_sign!(self, c, sign, |sign_key| {
			let key = self.get_newest_key().ok_or(SentcError::KeyNotFound)?;

			Ok(encrypt_raw_symmetric(&key.group_key, data, sign_key)?)
		})
	}

	pub async fn decrypt_raw(
		&mut self,
		head: &EncryptedHead,
		encrypted_data: &[u8],
		verify: bool,
		user_id: Option<&str>,
		c: &L1Cache,
	) -> Result<Vec<u8>, SentcError>
	{
		let key = group_key!(self, &head.id, c)?;

		let verify_key = get_verify_key_internally_for_decrypt(head, &self.base_url, &self.app_token, verify, user_id, c).await?;

		Ok(decrypt_raw_symmetric(
			&key.group_key,
			encrypted_data,
			head,
			verify_key.as_deref(),
		)?)
	}

	//______________________________________________________________________________________________
	//encrypt with aad

	pub async fn encrypt_raw_with_aad(&self, data: &[u8], aad: &[u8], sign: bool, c: &L1Cache) -> Result<(EncryptedHead, Vec<u8>), SentcError>
	{
		opt_sign!(self, c, sign, |sign_key| {
			let key = self.get_newest_key().ok_or(SentcError::KeyNotFound)?;

			Ok(encrypt_raw_symmetric_with_aad(&key.group_key, data, aad, sign_key)?)
		})
	}

	pub async fn decrypt_raw_with_aad(
		&mut self,
		head: &EncryptedHead,
		encrypted_data: &[u8],
		aad: &[u8],
		verify: bool,
		user_id: Option<&str>,
		c: &L1Cache,
	) -> Result<Vec<u8>, SentcError>
	{
		let key = group_key!(self, &head.id, c)?;

		let verify_key = get_verify_key_internally_for_decrypt(head, &self.base_url, &self.app_token, verify, user_id, c).await?;

		Ok(decrypt_raw_symmetric_with_aad(
			&key.group_key,
			encrypted_data,
			head,
			aad,
			verify_key.as_deref(),
		)?)
	}

	//______________________________________________________________________________________________

	pub async fn encrypt(&self, data: &[u8], sign: bool, c: &L1Cache) -> Result<Vec<u8>, SentcError>
	{
		opt_sign!(self, c, sign, |sign_key| {
			let key = self.get_newest_key().ok_or(SentcError::KeyNotFound)?;

			Ok(encrypt_symmetric(&key.group_key, data, sign_key)?)
		})
	}

	pub async fn decrypt(&mut self, data: &[u8], verify: bool, user_id: Option<&str>, c: &L1Cache) -> Result<Vec<u8>, SentcError>
	{
		let (head, data) = split_head_and_encrypted_data(data)?;

		self.decrypt_raw(&head, data, verify, user_id, c).await
	}

	//______________________________________________________________________________________________

	pub async fn encrypt_with_aad(&self, data: &[u8], aad: &[u8], sign: bool, c: &L1Cache) -> Result<Vec<u8>, SentcError>
	{
		opt_sign!(self, c, sign, |sign_key| {
			let key = self.get_newest_key().ok_or(SentcError::KeyNotFound)?;

			Ok(encrypt_symmetric_with_aad(&key.group_key, data, aad, sign_key)?)
		})
	}

	pub async fn decrypt_with_aad(&mut self, data: &[u8], aad: &[u8], verify: bool, user_id: Option<&str>, c: &L1Cache)
		-> Result<Vec<u8>, SentcError>
	{
		let (head, data) = split_head_and_encrypted_data(data)?;

		self.decrypt_raw_with_aad(&head, data, aad, verify, user_id, c)
			.await
	}

	//______________________________________________________________________________________________
	//encrypt string

	pub async fn encrypt_string(&self, data: &str, sign: bool, c: &L1Cache) -> Result<String, SentcError>
	{
		opt_sign!(self, c, sign, |sign_key| {
			let key = self.get_newest_key().ok_or(SentcError::KeyNotFound)?;

			Ok(encrypt_string_symmetric(&key.group_key, data, sign_key)?)
		})
	}

	pub async fn decrypt_string(&mut self, data: &str, verify: bool, user_id: Option<&str>, c: &L1Cache) -> Result<String, SentcError>
	{
		let head = split_head_and_encrypted_string(data)?;

		let key = group_key!(self, &head.id, c)?;

		let verify_key = get_verify_key_internally_for_decrypt(&head, &self.base_url, &self.app_token, verify, user_id, c).await?;

		Ok(decrypt_string_symmetric(&key.group_key, data, verify_key.as_deref())?)
	}

	//______________________________________________________________________________________________
	//encrypt string with aad

	pub async fn encrypt_string_with_aad(&self, data: &str, aad: &str, sign: bool, c: &L1Cache) -> Result<String, SentcError>
	{
		opt_sign!(self, c, sign, |sign_key| {
			let key = self.get_newest_key().ok_or(SentcError::KeyNotFound)?;

			Ok(encrypt_string_symmetric_with_aad(
				&key.group_key,
				data,
				aad,
				sign_key,
			)?)
		})
	}

	pub async fn decrypt_string_with_aad(
		&mut self,
		data: &str,
		aad: &str,
		verify: bool,
		user_id: Option<&str>,
		c: &L1Cache,
	) -> Result<String, SentcError>
	{
		let head = split_head_and_encrypted_string(data)?;

		let key = group_key!(self, &head.id, c)?;

		let verify_key = get_verify_key_internally_for_decrypt(&head, &self.base_url, &self.app_token, verify, user_id, c).await?;

		Ok(decrypt_string_symmetric_with_aad(
			&key.group_key,
			data,
			aad,
			verify_key.as_deref(),
		)?)
	}

	//==============================================================================================
	//sym key

	pub async fn get_non_registered_key(&mut self, master_key_id: &str, server_output: &str, c: &L1Cache) -> Result<SymKeyFormatInt, SentcError>
	{
		let key = group_key!(self, master_key_id, c)?;

		Ok(done_fetch_sym_key(&key.group_key, server_output, true)?)
	}
}
