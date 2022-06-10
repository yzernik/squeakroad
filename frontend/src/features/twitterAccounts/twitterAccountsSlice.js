import {
  createSlice,
  createSelector,
  createAsyncThunk,
  createEntityAdapter,
} from '@reduxjs/toolkit'
import {
  getTwitterAccounts,
  createTwitterAccount,
  deleteTwitterAccount,
} from '../../api/client'

const initialState = {
  twitterAccountsStatus: 'idle',
  twitterAccounts: [],
  createTwitterAccountStatus: 'idle',
}


export const setDeleteTwitterAccount = createAsyncThunk(
  'twitterAccounts/setDeleteTwitterAccount',
  async (values) => {
    console.log('Deleting twitter account');
    let twitterAccountId = values.twitterAccountId;
    await deleteTwitterAccount(twitterAccountId);
    const response = await getTwitterAccounts();
    console.log(response);
    return response.getTwitterAccountsList();
  }
)

export const fetchTwitterAccounts = createAsyncThunk(
  'twitterAccounts/fetchTwitterAccounts',
  async () => {
    const response = await getTwitterAccounts();
    console.log(response);
    return response.getTwitterAccountsList();
  }
)

export const setCreateTwitterAccount = createAsyncThunk(
  'twitterAccounts/setCreateTwitterAccount',
  async (values) => {
    console.log('Creating twitter account');
    let twitterHandle = values.twitterHandle;
    let profileId = values.profileId;
    let bearerToken = values.bearerToken;
    const createResponse = await createTwitterAccount(
      twitterHandle,
      profileId,
      bearerToken,
    );
    console.log(createResponse);
    const response = await getTwitterAccounts();
    console.log(response);
    return response.getTwitterAccountsList();
  }
)


// const updatedProfileInArray = (profileArr, newProfile) => {
//   const currentIndex = profileArr.findIndex(profile => profile.getPubkey() === newProfile.getPubkey());
//   if (currentIndex != -1) {
//     profileArr[currentIndex] = newProfile;
//   }
// }


const twitterAccountsSlice = createSlice({
  name: 'twitterAccounts',
  initialState,
  reducers: {
    clearTwitterAccounts(state, action) {
      state.createTwitterAccountStatus = 'idle'
      state.twitterAccounts = [];
    },
  },
  extraReducers: (builder) => {
    builder
    .addCase(fetchTwitterAccounts.pending, (state, action) => {
      state.twitterAccountsStatus = 'loading'
    })
    .addCase(fetchTwitterAccounts.fulfilled, (state, action) => {
      const newTwitterAccounts = action.payload;
      state.twitterAccounts = newTwitterAccounts;
      state.twitterAccountsStatus = 'idle'
    })
    .addCase(setCreateTwitterAccount.pending, (state, action) => {
      console.log('setCreateTwitterAccount pending');
      state.createTwitterAccountStatus = 'loading'
    })
    .addCase(setCreateTwitterAccount.fulfilled, (state, action) => {
      console.log('setCreateTwitterAccount fulfilled');
      console.log(action);
      const newTwitterAccounts = action.payload;
      state.twitterAccounts = newTwitterAccounts;
      state.twitterAccountsStatus = 'idle'
      state.createTwitterAccountStatus = 'idle';
    })
    .addCase(setDeleteTwitterAccount.fulfilled, (state, action) => {
      console.log(action);
      const newTwitterAccounts = action.payload;
      state.twitterAccounts = newTwitterAccounts;
      state.twitterAccountsStatus = 'idle'

    })
  },
})

export const {
  clearTwitterAccounts,
} = twitterAccountsSlice.actions

export default twitterAccountsSlice.reducer

export const selectTwitterAccountsStatus = state => state.twitterAccounts.twitterAccountsStatus

export const selectTwitterAccounts = state => state.twitterAccounts.twitterAccounts

export const selectCreateTwitterAccountStatus = state => state.twitterAccounts.createTwitterAccountStatus
