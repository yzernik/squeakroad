import {
  createSlice,
  createSelector,
  createAsyncThunk,
  createEntityAdapter,
} from '@reduxjs/toolkit'
import {
  getExternalAddress,
} from '../../api/client'

const initialState = {
  externalAddress: null
}

// Thunk functions
export const fetchExternalAddress = createAsyncThunk(
  'externalAddress/fetchExternalAddress',
  async () => {
    console.log('Fetching external address');
    const response = await getExternalAddress();
    console.log(response);
    return response.getPeerAddress();
  }
)

const externalAddressSlice = createSlice({
  name: 'externalAddress',
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder
    .addCase(fetchExternalAddress.fulfilled, (state, action) => {
      console.log(action);
      const externalAddress = action.payload;
      state.externalAddress = externalAddress;
    })
  },
})

export default externalAddressSlice.reducer

export const selectExternalAddress = state => state.externalAddress.externalAddress
