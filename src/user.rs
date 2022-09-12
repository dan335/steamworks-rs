use super::*;
#[cfg(test)]
use serial_test_derive::serial;

/// Access to the steam user interface
pub struct User<Manager> {
    pub(crate) user: *mut sys::ISteamUser,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> User<Manager> {
    /// Returns the steam id of the current user
    pub fn steam_id(&self) -> SteamId {
        unsafe { SteamId(sys::SteamAPI_ISteamUser_GetSteamID(self.user)) }
    }

    /// Returns the level of the current user
    pub fn level(&self) -> u32 {
        unsafe { sys::SteamAPI_ISteamUser_GetPlayerSteamLevel(self.user) as u32 }
    }

    /// Retrieve an authentication session ticket that can be sent
    /// to an entity that wishes to verify you.
    ///
    /// This ticket should not be reused.
    ///
    /// When creating ticket for use by the web API you should wait
    /// for the `AuthSessionTicketResponse` event before trying to
    /// use the ticket.
    ///
    /// When the multiplayer session terminates you must call
    /// `cancel_authentication_ticket`
    pub fn authentication_session_ticket(&self) -> (AuthTicket, Vec<u8>) {
        unsafe {
            let mut ticket = vec![0; 1024];
            let mut ticket_len = 0;
            let auth_ticket = sys::SteamAPI_ISteamUser_GetAuthSessionTicket(
                self.user,
                ticket.as_mut_ptr() as *mut _,
                1024,
                &mut ticket_len,
            );
            ticket.truncate(ticket_len as usize);
            (AuthTicket(auth_ticket), ticket)
        }
    }

    /// Cancels an authentication session ticket received from
    /// `authentication_session_ticket`.
    ///
    /// This should be called when you are no longer playing with
    /// the specified entity.
    pub fn cancel_authentication_ticket(&self, ticket: AuthTicket) {
        unsafe {
            sys::SteamAPI_ISteamUser_CancelAuthTicket(self.user, ticket.0);
        }
    }

    /// Authenticate the ticket from the steam ID to make sure it is
    /// valid and not reused.
    ///
    /// A `ValidateAuthTicketResponse` callback will be fired if
    /// the entity goes offline or cancels the ticket.
    ///
    /// When the multiplayer session terminates you must call
    /// `end_authentication_session`
    pub fn begin_authentication_session(
        &self,
        user: SteamId,
        ticket: &[u8],
    ) -> Result<(), AuthSessionError> {
        unsafe {
            let res = sys::SteamAPI_ISteamUser_BeginAuthSession(
                self.user,
                ticket.as_ptr() as *const _,
                ticket.len() as _,
                user.0,
            );
            Err(match res {
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultOK => return Ok(()),
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultInvalidTicket => {
                    AuthSessionError::InvalidTicket
                }
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultDuplicateRequest => {
                    AuthSessionError::DuplicateRequest
                }
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultInvalidVersion => {
                    AuthSessionError::InvalidVersion
                }
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultGameMismatch => {
                    AuthSessionError::GameMismatch
                }
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultExpiredTicket => {
                    AuthSessionError::ExpiredTicket
                }
                _ => unreachable!(),
            })
        }
    }

    /// Ends an authentication session that was started with
    /// `begin_authentication_session`.
    ///
    /// This should be called when you are no longer playing with
    /// the specified entity.
    pub fn end_authentication_session(&self, user: SteamId) {
        unsafe {
            sys::SteamAPI_ISteamUser_EndAuthSession(self.user, user.0);
        }
    }

    /// Checks to see if there is captured audio data available
    /// from GetVoice, and gets the size of the data.
    ///
    /// Most applications will only use compressed data and should
    /// ignore the other parameters, which exist primarily for
    /// backwards compatibility. See GetVoice for further explanation
    /// of "uncompressed" data.
    pub fn get_available_voice(
        &self,
        pcb_compressed: &mut u32
    ) -> Result<(), VoiceResult> {
        unsafe {
            let res = sys::SteamAPI_ISteamUser_GetAvailableVoice(
                self.user,
                pcb_compressed as *mut _,
            );
            Err(match res {
                sys::EVoiceResult::k_EVoiceResultOK => return Ok(()),
                sys::EVoiceResult::k_EVoiceResultNotInitialized => {
                    VoiceResult::NotInitialized
                },
                sys::EVoiceResult::k_EVoiceResultNotRecording => {
                    VoiceResult::NotRecording
                },
                sys::EVoiceResult::k_EVoiceResultNoData => {
                    VoiceResult::NoData
                },
                sys::EVoiceResult::k_EVoiceResultBufferTooSmall => {
                    VoiceResult::BufferTooSmall
                },
                sys::EVoiceResult::k_EVoiceResultDataCorrupted => {
                    VoiceResult::DataCorrupted
                },
                sys::EVoiceResult::k_EVoiceResultRestricted => {
                    VoiceResult::Restricted
                },
                _ => unreachable!(),
            })
        }
    }

    /// Read captured audio data from the microphone buffer.
    ///
    /// The compressed data can be transmitted by your application
    /// and decoded back into raw audio data using DecompressVoice
    /// on the other side. The compressed data provided is in an
    /// arbitrary format and is not meant to be played directly.
    ///
    /// This should be called once per frame, and at worst no more
    /// than four times a second to keep the microphone input delay
    /// as low as possible. Calling this any less may result in gaps
    /// in the returned stream.
    ///
    /// It is recommended that you pass in an 8 kilobytes or larger
    /// destination buffer for compressed audio. Static buffers are
    /// recommended for performance reasons. However, if you would
    /// like to allocate precisely the right amount of space for a
    /// buffer before each call you may use GetAvailableVoice to find
    /// out how much data is available to be read.
    ///
    /// NOTE: "Uncompressed" audio is a deprecated feature and should
    /// not be used by most applications. It is raw single-channel
    /// 16-bit PCM wave data which may have been run through preprocessing
    /// filters and/or had silence removed, so the uncompressed audio
    /// could have a shorter duration than you expect. There may be no
    /// data at all during long periods of silence. Also, fetching
    /// uncompressed audio will cause GetVoice to discard any leftover
    /// compressed audio, so you must fetch both types at once. Finally,
    /// GetAvailableVoice is not precisely accurate when the uncompressed
    /// size is requested. So if you really need to use uncompressed
    /// audio, you should call GetVoice frequently with two very large
    /// (20KiB+) output buffers instead of trying to allocate
    /// perfectly-sized buffers. But most applications should ignore
    /// all of these details and simply leave the "uncompressed"
    /// parameters as NULL/0.
    pub fn get_voice(
        &self,
        p_dest_buffer: &mut [u8],
        n_bytes_written: &mut u32
    ) -> Result<(), VoiceResult> {
        unsafe {
            let res = sys::SteamAPI_ISteamUser_GetVoice(
                self.user,
                true,
                //p_dest_buffer.as_ptr() as *mut c_void,
                p_dest_buffer.as_mut_ptr() as *mut c_void,
                p_dest_buffer.len() as _,
                n_bytes_written as *mut _,
            );
            Err(match res {
                sys::EVoiceResult::k_EVoiceResultOK => return Ok(()),
                sys::EVoiceResult::k_EVoiceResultNotInitialized => {
                    VoiceResult::NotInitialized
                },
                sys::EVoiceResult::k_EVoiceResultNotRecording => {
                    VoiceResult::NotRecording
                },
                sys::EVoiceResult::k_EVoiceResultNoData => {
                    VoiceResult::NoData
                },
                sys::EVoiceResult::k_EVoiceResultBufferTooSmall => {
                    VoiceResult::BufferTooSmall
                },
                sys::EVoiceResult::k_EVoiceResultDataCorrupted => {
                    VoiceResult::DataCorrupted
                },
                sys::EVoiceResult::k_EVoiceResultRestricted => {
                    VoiceResult::Restricted
                },
                _ => unreachable!(),
            })
        }
    }

    /// Decodes the compressed voice data returned by GetVoice.
    ///
    /// The output data is raw single-channel 16-bit PCM audio.
    /// The decoder supports any sample rate from 11025 to 48000.
    /// See GetVoiceOptimalSampleRate for more information.
    ///
    /// It is recommended that you start with a 20KiB buffer and
    /// then reallocate as necessary.
    pub fn decompress_voice(
        &self, p_compressed: &[u8],
        p_dest_buffer: &mut [u8],
        n_bytes_written: &mut u32,
        n_desired_sample_rate: u32
    ) -> Result<(), VoiceResult> {
        unsafe {
            let res = sys::SteamAPI_ISteamUser_DecompressVoice(
                self.user,
                p_compressed.as_ptr() as *const c_void,
                p_compressed.len() as u32,
                p_dest_buffer.as_ptr() as *mut c_void,
                p_dest_buffer.len() as u32,
                n_bytes_written as *mut u32,
                n_desired_sample_rate
            );
            Err(match res {
                sys::EVoiceResult::k_EVoiceResultOK => return Ok(()),
                sys::EVoiceResult::k_EVoiceResultNotInitialized => {
                    VoiceResult::NotInitialized
                },
                sys::EVoiceResult::k_EVoiceResultNotRecording => {
                    VoiceResult::NotRecording
                },
                sys::EVoiceResult::k_EVoiceResultNoData => {
                    VoiceResult::NoData
                },
                sys::EVoiceResult::k_EVoiceResultBufferTooSmall => {
                    VoiceResult::BufferTooSmall
                },
                sys::EVoiceResult::k_EVoiceResultDataCorrupted => {
                    VoiceResult::DataCorrupted
                },
                sys::EVoiceResult::k_EVoiceResultRestricted => {
                    VoiceResult::Restricted
                },
                _ => unreachable!(),
            })
        }
    }

    /// Starts voice recording.
    ///
    /// Once started, use GetAvailableVoice and GetVoice to get
    /// the data, and then call StopVoiceRecording when the user
    /// has released their push-to-talk hotkey or the game session
    /// has completed.
    pub fn start_voice_recording(&self) {
        unsafe {
            return sys::SteamAPI_ISteamUser_StartVoiceRecording(self.user);
        }
    }

    /// Stops voice recording.
    ///
    /// Because people often release push-to-talk keys early, the
    /// system will keep recording for a little bit after this function
    /// is called. As such, GetVoice should continue to be called
    /// until it returns k_EVoiceResultNotRecording, only then will
    /// voice recording be stopped.
    pub fn stop_voice_recording(&self) {
        unsafe {
            return sys::SteamAPI_ISteamUser_StopVoiceRecording(self.user);
        }
    }

    /// Gets the native sample rate of the Steam voice decoder.
    ///
    /// Using this sample rate for DecompressVoice will perform the
    /// least CPU processing. However, the final audio quality will
    /// depend on how well the audio device (and/or your application's
    /// audio output SDK) deals with lower sample rates. You may find
    /// that you get the best audio output quality when you ignore this
    /// function and use the native sample rate of your audio output
    /// device, which is usually 48000 or 44100.
    pub fn get_voice_optimal_sample_rate(&self) -> u32 {
        unsafe {
            return sys::SteamAPI_ISteamUser_GetVoiceOptimalSampleRate(self.user);
        }
    }
}

/// Errors from `begin_authentication_session`
#[derive(Debug, Error)]
pub enum AuthSessionError {
    /// The ticket is invalid
    #[error("invalid ticket")]
    InvalidTicket,
    /// A ticket has already been submitted for this steam ID
    #[error("duplicate ticket request")]
    DuplicateRequest,
    /// The ticket is from an incompatible interface version
    #[error("incompatible interface version")]
    InvalidVersion,
    /// The ticket is not for this game
    #[error("incorrect game for ticket")]
    GameMismatch,
    /// The ticket has expired
    #[error("ticket has expired")]
    ExpiredTicket,
}

#[test]
#[serial]
fn test() {
    let (client, single) = Client::init().unwrap();
    let user = client.user();

    let _cb = client
        .register_callback(|v: AuthSessionTicketResponse| println!("Got response: {:?}", v.result));
    let _cb = client.register_callback(|v: ValidateAuthTicketResponse| println!("{:?}", v));

    let id = user.steam_id();
    let (auth, ticket) = user.authentication_session_ticket();

    println!("{:?}", user.begin_authentication_session(id, &ticket));

    for _ in 0..20 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    println!("END");

    user.cancel_authentication_ticket(auth);

    for _ in 0..20 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    user.end_authentication_session(id);
}



/// A handle for an authentication ticket that can be used to cancel
/// it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthTicket(pub(crate) sys::HAuthTicket);

/// Called when generating a authentication session ticket.
///
/// This can be used to verify the ticket was created successfully.
pub struct AuthSessionTicketResponse {
    /// The ticket in question
    pub ticket: AuthTicket,
    /// The result of generating the ticket
    pub result: SResult<()>,
}

unsafe impl Callback for AuthSessionTicketResponse {
    const ID: i32 = 163;
    const SIZE: i32 = ::std::mem::size_of::<sys::GetAuthSessionTicketResponse_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::GetAuthSessionTicketResponse_t);
        AuthSessionTicketResponse {
            ticket: AuthTicket(val.m_hAuthTicket),
            result: if val.m_eResult == sys::EResult::k_EResultOK {
                Ok(())
            } else {
                Err(val.m_eResult.into())
            },
        }
    }
}

/// Called when an authentication ticket has been
/// validated.
#[derive(Debug)]
pub struct ValidateAuthTicketResponse {
    /// The steam id of the entity that provided the ticket
    pub steam_id: SteamId,
    /// The result of the validation
    pub response: Result<(), AuthSessionValidateError>,
    /// The steam id of the owner of the game. Differs from
    /// `steam_id` if the game is borrowed.
    pub owner_steam_id: SteamId,
}

unsafe impl Callback for ValidateAuthTicketResponse {
    const ID: i32 = 143;
    const SIZE: i32 = ::std::mem::size_of::<sys::ValidateAuthTicketResponse_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::ValidateAuthTicketResponse_t);
        ValidateAuthTicketResponse {
            steam_id: SteamId(val.m_SteamID.m_steamid.m_unAll64Bits),
            owner_steam_id: SteamId(val.m_OwnerSteamID.m_steamid.m_unAll64Bits),
            response: match val.m_eAuthSessionResponse {
                sys::EAuthSessionResponse::k_EAuthSessionResponseOK => Ok(()),
                sys::EAuthSessionResponse::k_EAuthSessionResponseUserNotConnectedToSteam => {
                    Err(AuthSessionValidateError::UserNotConnectedToSteam)
                }
                sys::EAuthSessionResponse::k_EAuthSessionResponseNoLicenseOrExpired => {
                    Err(AuthSessionValidateError::NoLicenseOrExpired)
                }
                sys::EAuthSessionResponse::k_EAuthSessionResponseVACBanned => {
                    Err(AuthSessionValidateError::VACBanned)
                }
                sys::EAuthSessionResponse::k_EAuthSessionResponseLoggedInElseWhere => {
                    Err(AuthSessionValidateError::LoggedInElseWhere)
                }
                sys::EAuthSessionResponse::k_EAuthSessionResponseVACCheckTimedOut => {
                    Err(AuthSessionValidateError::VACCheckTimedOut)
                }
                sys::EAuthSessionResponse::k_EAuthSessionResponseAuthTicketCanceled => {
                    Err(AuthSessionValidateError::AuthTicketCancelled)
                }
                sys::EAuthSessionResponse::k_EAuthSessionResponseAuthTicketInvalidAlreadyUsed => {
                    Err(AuthSessionValidateError::AuthTicketInvalidAlreadyUsed)
                }
                sys::EAuthSessionResponse::k_EAuthSessionResponseAuthTicketInvalid => {
                    Err(AuthSessionValidateError::AuthTicketInvalid)
                }
                sys::EAuthSessionResponse::k_EAuthSessionResponsePublisherIssuedBan => {
                    Err(AuthSessionValidateError::PublisherIssuedBan)
                }
                _ => unreachable!(),
            },
        }
    }
}

/// Called when a connection to the Steam servers is made.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamServersConnected;

unsafe impl Callback for SteamServersConnected {
    const ID: i32 = 101;
    const SIZE: i32 = ::std::mem::size_of::<sys::SteamServersConnected_t>() as i32;

    unsafe fn from_raw(_: *mut c_void) -> Self {
        SteamServersConnected
    }
}

/// Called when the connection to the Steam servers is lost.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamServersDisconnected {
    /// The reason we were disconnected from the Steam servers
    pub reason: SteamError,
}

unsafe impl Callback for SteamServersDisconnected {
    const ID: i32 = 103;
    const SIZE: i32 = ::std::mem::size_of::<sys::SteamServersDisconnected_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::SteamServersDisconnected_t);
        SteamServersDisconnected {
            reason: val.m_eResult.into(),
        }
    }
}

/// Called when the connection to the Steam servers fails.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamServerConnectFailure {
    /// The reason we failed to connect to the Steam servers
    pub reason: SteamError,
    /// Whether we are still retrying the connection.
    pub still_retrying: bool,
}

unsafe impl Callback for SteamServerConnectFailure {
    const ID: i32 = 102;
    const SIZE: i32 = ::std::mem::size_of::<sys::SteamServerConnectFailure_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::SteamServerConnectFailure_t);
        SteamServerConnectFailure {
            reason: val.m_eResult.into(),
            still_retrying: val.m_bStillRetrying,
        }
    }
}

/// Errors from `ValidateAuthTicketResponse`
#[derive(Debug, Error)]
pub enum AuthSessionValidateError {
    /// The user in question is not connected to steam
    #[error("user not connected to steam")]
    UserNotConnectedToSteam,
    /// The license has expired
    #[error("the license has expired")]
    NoLicenseOrExpired,
    /// The user is VAC banned from the game
    #[error("the user is VAC banned from this game")]
    VACBanned,
    /// The user has logged in elsewhere and the session
    /// has been disconnected
    #[error("the user is logged in elsewhere")]
    LoggedInElseWhere,
    /// VAC has been unable to perform anti-cheat checks on this
    /// user
    #[error("VAC check timed out")]
    VACCheckTimedOut,
    /// The ticket has been cancelled by the issuer
    #[error("the authentication ticket has been cancelled")]
    AuthTicketCancelled,
    /// The ticket has already been used
    #[error("the authentication ticket has already been used")]
    AuthTicketInvalidAlreadyUsed,
    /// The ticket is not from a user instance currently connected
    /// to steam
    #[error("the authentication ticket is invalid")]
    AuthTicketInvalid,
    /// The user is banned from the game (not VAC)
    #[error("the user is banned")]
    PublisherIssuedBan,
}

#[derive(Debug, Error)]
pub enum VoiceResult {
    // The Steam Voice interface has not been initialized.
    #[error("the steam voice interface has not been initialized")]
    NotInitialized,
    // Steam Voice is not currently recording.
    #[error("steam voice is not currently recording")]
    NotRecording,
    // There is no voice data available.
    #[error("there is no voice data available")]
    NoData,
    // The provided buffer is too small to receive the data.
    #[error("the provided buffer is too small to receive the data")]
    BufferTooSmall,
    // The voice data has been corrupted.
    #[error("the voice data has been corrupted")]
    DataCorrupted,
    // The user is chat restricted.
    #[error("the user is chat restricted")]
    Restricted,
}