import {
  createSlice,
  createSelector,
  createAsyncThunk,
  createEntityAdapter,
} from '@reduxjs/toolkit'
import {
  getProfile,
  getProfileByPubkey,
  setProfileFollowing,
  deleteProfile,
  getSigningProfiles,
  getContactProfiles,
  createContactProfile,
  createSigningProfile,
  importSigningProfile,
  renameProfile,
  changeProfileImage,
  clearProfileImage,
  getPrivateKey,
} from '../../api/client'

const initialState = {
  currentProfileStatus: 'idle',
  currentProfile: null,
  signingProfilesStatus: 'idle',
  signingProfiles: [],
  contactProfilesStatus: 'idle',
  contactProfiles: [],
  createContactProfileStatus: 'idle',
  createSigningProfileStatus: 'idle',
  importSigningProfileStatus: 'idle',
  exportPrivateKeyStatus: 'idle',
}

// Thunk functions
export const fetchProfile = createAsyncThunk(
  'profile/fetchProfile',
  async (pubkey) => {
    console.log('Fetching profile');
    const response = await getProfileByPubkey(pubkey);
    console.log(response);
    return response.getSqueakProfile();
  }
)

// Use profile id for now. In the future, change RPC to accept pubkey.
export const setFollowProfile = createAsyncThunk(
  'profile/setFollowProfile',
  async (id) => {
    console.log('Following profile');
    await setProfileFollowing(id, true);
    const response = await getProfile(id);
    return response.getSqueakProfile();
  }
)

// Use profile id for now. In the future, change RPC to accept pubkey.
export const setUnfollowProfile = createAsyncThunk(
  'profile/setUnfollowProfile',
  async (id) => {
    console.log('Unfollowing profile');
    await setProfileFollowing(id, false);
    const response = await getProfile(id);
    return response.getSqueakProfile();
  }
)

// Use profile id for now. In the future, change RPC to accept pubkey.
export const setDeleteProfile = createAsyncThunk(
  'profile/setDeleteProfile',
  async (values) => {
    console.log('Deleting profile');
    await deleteProfile(values.profileId);
    return null;
  }
)

// Use profile id for now. In the future, change RPC to accept pubkey.
export const setRenameProfile = createAsyncThunk(
  'profile/setRenameProfile',
  async (values) => {
    console.log('Renaming profile');
    let profileId = values.profileId;
    let profileName = values.profileName;
    await renameProfile(profileId, profileName);
    const response = await getProfile(profileId);
    return response.getSqueakProfile();
  }
)

// Use profile id for now. In the future, change RPC to accept pubkey.
export const setProfileImage = createAsyncThunk(
  'profile/setProfileImage',
  async (values) => {
    console.log('Changing image for profile');
    let profileId = values.profileId;
    let imageBase64 = values.imageBase64;
    await changeProfileImage(profileId, imageBase64);
    const response = await getProfile(profileId);
    return response.getSqueakProfile();
  }
)

// Use profile id for now. In the future, change RPC to accept pubkey.
export const setClearProfileImage = createAsyncThunk(
  'profile/setClearProfileImage',
  async (values) => {
    console.log('Clearing image for profile');
    let profileId = values.profileId;
    await clearProfileImage(profileId);
    const response = await getProfile(profileId);
    return response.getSqueakProfile();
  }
)

export const fetchSigningProfiles = createAsyncThunk(
  'profiles/fetchSigningProfiles',
  async () => {
    const response = await getSigningProfiles();
    return response.getSqueakProfilesList();
  }
)

export const fetchContactProfiles = createAsyncThunk(
  'profiles/fetchContactProfiles',
  async () => {
    const response = await getContactProfiles();
    return response.getSqueakProfilesList();
  }
)

export const setCreateContactProfile = createAsyncThunk(
  'profiles/createContactProfile',
  async (values) => {
    console.log('Creating contact profile');
    let profileName = values.profileName;
    let pubkey = values.pubkey;
    const createResponse = await createContactProfile(profileName, pubkey);
    console.log(createResponse);
    const response = await getProfile(createResponse.getProfileId());
    console.log(response);
    return response.getSqueakProfile().getPubkey();
  }
)

export const setCreateSigningProfile = createAsyncThunk(
  'profiles/createSigningProfile',
  async (values) => {
    console.log('Creating signing profile');
    let profileName = values.profileName;
    const createResponse = await createSigningProfile(profileName);
    console.log(createResponse);
    const response = await getProfile(createResponse.getProfileId());
    console.log(response);
    return response.getSqueakProfile().getPubkey();
  }
)

export const setImportSigningProfile = createAsyncThunk(
  'profiles/importSigningProfile',
  async (values) => {
    console.log('Importing signing profile');
    let profileName = values.profileName;
    let privateKey = values.privateKey;
    const createResponse = await importSigningProfile(profileName, privateKey);
    console.log(createResponse);
    const response = await getProfile(createResponse.getProfileId());
    console.log(response);
    return response.getSqueakProfile().getPubkey();
  }
)

export const getProfilePrivateKey = createAsyncThunk(
  'profiles/getProfilePrivateKey',
  async (values) => {
    console.log('Exporting profile private key');
    let profileId = values.profileId;
    const response = await getPrivateKey(profileId);
    console.log(response);
    return response.getPrivateKey();
  }
)

const updatedProfileInArray = (profileArr, newProfile) => {
  const currentIndex = profileArr.findIndex(profile => profile.getPubkey() === newProfile.getPubkey());
  if (currentIndex != -1) {
    profileArr[currentIndex] = newProfile;
  }
}


const profilesSlice = createSlice({
  name: 'profiles',
  initialState,
  reducers: {
    clearAll(state, action) {
      console.log('Clear profile and other data.');
      state.currentProfileStatus = 'idle';
      state.currentProfile = null;
    },
    clearSigningProfiles(state, action) {
      state.signingProfilesStatus = 'idle'
      state.signingProfiles = [];
    },
    clearContactProfiles(state, action) {
      state.contactProfilesStatus = 'idle'
      state.contactProfiles = [];
    },
  },
  extraReducers: (builder) => {
    builder
    .addCase(fetchProfile.pending, (state, action) => {
      state.currentProfileStatus = 'loading'
    })
    .addCase(fetchProfile.fulfilled, (state, action) => {
      console.log(action);
      const newProfile = action.payload;
      state.currentProfile = newProfile;
      state.currentProfileStatus = 'idle';
    })
    .addCase(setFollowProfile.fulfilled, (state, action) => {
      console.log(action);
      const newProfile = action.payload;
      if (state.currentProfile && state.currentProfile.getPubkey() === newProfile.getPubkey()) {
        state.currentProfile = newProfile;
      }
      updatedProfileInArray(state.signingProfiles, newProfile);
      updatedProfileInArray(state.contactProfiles, newProfile);
    })
    .addCase(setUnfollowProfile.fulfilled, (state, action) => {
      console.log(action);
      const newProfile = action.payload;
      if (state.currentProfile && state.currentProfile.getPubkey() === newProfile.getPubkey()) {
        state.currentProfile = newProfile;
      }
      updatedProfileInArray(state.signingProfiles, newProfile);
      updatedProfileInArray(state.contactProfiles, newProfile);
    })
    .addCase(setRenameProfile.fulfilled, (state, action) => {
      console.log(action);
      const newProfile = action.payload;
      if (state.currentProfile && state.currentProfile.getPubkey() === newProfile.getPubkey()) {
        state.currentProfile = newProfile;
      }
      updatedProfileInArray(state.signingProfiles, newProfile);
      updatedProfileInArray(state.contactProfiles, newProfile);
    })
    .addCase(setProfileImage.fulfilled, (state, action) => {
      console.log(action);
      const newProfile = action.payload;
      if (state.currentProfile && state.currentProfile.getPubkey() === newProfile.getPubkey()) {
        state.currentProfile = newProfile;
      }
      updatedProfileInArray(state.signingProfiles, newProfile);
      updatedProfileInArray(state.contactProfiles, newProfile);
    })
    .addCase(setClearProfileImage.fulfilled, (state, action) => {
      console.log(action);
      const newProfile = action.payload;
      if (state.currentProfile && state.currentProfile.getPubkey() === newProfile.getPubkey()) {
        state.currentProfile = newProfile;
      }
      updatedProfileInArray(state.signingProfiles, newProfile);
      updatedProfileInArray(state.contactProfiles, newProfile);
    })
    .addCase(setDeleteProfile.fulfilled, (state, action) => {
      console.log(action);
      const deletedProfileId = action.meta.arg.profileId;
      console.log(deletedProfileId);
      if (state.currentProfile && state.currentProfile.getProfileId() === deletedProfileId) {
        state.currentProfile = null;
      }
    })
    .addCase(fetchSigningProfiles.pending, (state, action) => {
      state.signingProfilesStatus = 'loading'
    })
    .addCase(fetchSigningProfiles.fulfilled, (state, action) => {
      const newSigningProfiles = action.payload;
      state.signingProfiles = newSigningProfiles;
      state.signingProfilesStatus = 'idle'
    })
    .addCase(fetchContactProfiles.pending, (state, action) => {
      state.contactProfilesStatus = 'loading'
    })
    .addCase(fetchContactProfiles.fulfilled, (state, action) => {
      const newContactProfiles = action.payload;
      state.contactProfiles = newContactProfiles;
      state.contactProfilesStatus = 'idle'
    })
    .addCase(setCreateContactProfile.pending, (state, action) => {
      console.log('setCreateContactProfile pending');
      state.createContactProfileStatus = 'loading'
    })
    .addCase(setCreateContactProfile.fulfilled, (state, action) => {
      console.log('setCreateContactProfile fulfilled');
      console.log(action);
      const newSqueakHash = action.payload;
      state.createContactProfileStatus = 'idle';
      console.log('Go to new profile');
    })
    .addCase(setCreateSigningProfile.pending, (state, action) => {
      console.log('setCreateSigningProfile pending');
      state.createSigningProfileStatus = 'loading'
    })
    .addCase(setCreateSigningProfile.fulfilled, (state, action) => {
      console.log('setCreateSigningProfile fulfilled');
      console.log(action);
      const newSqueakHash = action.payload;
      state.createSigningProfileStatus = 'idle';
      console.log('Go to new profile');
    })
    .addCase(setImportSigningProfile.pending, (state, action) => {
      console.log('setImportSigningProfile pending');
      state.importSigningProfileStatus = 'loading'
    })
    .addCase(setImportSigningProfile.fulfilled, (state, action) => {
      console.log('setImportSigningProfile fulfilled');
      console.log(action);
      const newSqueakHash = action.payload;
      state.importSigningProfileStatus = 'idle';
      console.log('Go to new profile');
    })
    .addCase(getProfilePrivateKey.pending, (state, action) => {
      console.log('getProfilePrivateKey pending');
      state.exportPrivateKeyStatus = 'loading'
    })
    .addCase(getProfilePrivateKey.fulfilled, (state, action) => {
      console.log('getProfilePrivateKey fulfilled');
      console.log(action);
      state.exportPrivateKeyStatus = 'idle';
    })
  },
})

export const {
  clearAll,
  clearSigningProfiles,
  clearContactProfiles,
} = profilesSlice.actions

export default profilesSlice.reducer

export const selectCurrentProfile = state => state.profiles.currentProfile

export const selectCurrentProfileStatus = state => state.profiles.currentProfileStatus

export const selectSigningProfiles = state => state.profiles.signingProfiles

export const selectSigningProfilesStatus = state => state.profiles.signingProfilesStatus

export const selectContactProfiles = state => state.profiles.contactProfiles

export const selectContactProfilesStatus = state => state.profiles.contactProfilesStatus

export const selectCreateContactProfileStatus = state => state.createContactProfile.createContactProfileStatus

export const selectCreateSigningProfileStatus = state => state.createSigningProfile.createSigningProfileStatus

export const selectImportSigningProfileStatus = state => state.importSigningProfile.importSigningProfileStatus
