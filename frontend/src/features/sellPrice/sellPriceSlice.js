import {
  createSlice,
  createSelector,
  createAsyncThunk,
  createEntityAdapter,
} from '@reduxjs/toolkit'
import {
  getSellPrice,
  updateSellPrice,
  clearSellPrice,
} from '../../api/client'

const initialState = {
  sellPriceInfo: null,
}

// Thunk functions
export const fetchSellPrice = createAsyncThunk(
  'sellPrice/fetchSellPrice',
  async () => {
    console.log('Fetching sellPrice');
    const response = await getSellPrice();
    console.log(response);
    return response;
  }
)

export const setSellPrice = createAsyncThunk(
  'sellPrice/setSellPrice',
  async (sellPriceMsat) => {
    console.log('Updating sellPrice');
    await updateSellPrice(sellPriceMsat);
    const response = await getSellPrice();
    console.log(response);
    return response;
  }
)

export const setClearSellPrice = createAsyncThunk(
  'sellPrice/setClearSellPrice',
  async (sellPriceMsat) => {
    console.log('Clearing sellPrice');
    await clearSellPrice();
    const response = await getSellPrice();
    console.log(response);
    return response;
  }
)

const sellPriceSlice = createSlice({
  name: 'sellPrice',
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder
    .addCase(fetchSellPrice.fulfilled, (state, action) => {
      console.log(action);
      const sellPriceInfo = action.payload;
      state.sellPriceInfo = sellPriceInfo;
    })
    .addCase(setSellPrice.fulfilled, (state, action) => {
      console.log(action);
      const sellPriceInfo = action.payload;
      state.sellPriceInfo = sellPriceInfo;
    })
    .addCase(setClearSellPrice.fulfilled, (state, action) => {
      console.log(action);
      const sellPriceInfo = action.payload;
      state.sellPriceInfo = sellPriceInfo;
    })
  },
})

export default sellPriceSlice.reducer

export const selectSellPriceInfo = state => state.sellPrice.sellPriceInfo

// export const selectSellPriceUsingOverride = state => state.sellPrice.sellPriceInfo && state.sellPrice.sellPriceInfo.getPriceMsatIsSet()
//
// export const selectSellPriceOverride = state => state.sellPrice.sellPriceInfo && state.sellPrice.sellPriceInfo.getPriceMsat()
//
// export const selectSellPriceDefault = state => state.sellPrice.sellPriceInfo && state.sellPrice.sellPriceInfo.getDefaultPriceMsat()
