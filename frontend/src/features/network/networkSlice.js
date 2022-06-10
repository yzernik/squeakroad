import {
  createSlice,
  createSelector,
  createAsyncThunk,
  createEntityAdapter,
} from '@reduxjs/toolkit'
import { client, getNetwork } from '../../api/client'

const initialState = {
  network: null
}

// Thunk functions
export const fetchNetwork = createAsyncThunk(
  'network/fetchNetwork',
  async () => {
    console.log('Fetching network');
    const response = await getNetwork();
    console.log(response);
    return response.getNetwork();
  }
)

const networkSlice = createSlice({
  name: 'network',
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder
    .addCase(fetchNetwork.fulfilled, (state, action) => {
      console.log(action);
      const network = action.payload;
      state.network = network;
    })
  },
})

export default networkSlice.reducer

export const selectNetwork = state => state.network.network
