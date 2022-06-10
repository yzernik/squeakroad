import {
  createSlice,
  createSelector,
  createAsyncThunk,
  createEntityAdapter,
} from '@reduxjs/toolkit'
import {
  getSentPayments,
  getSentPaymentsForSqueak,
  getSentPaymentsForPubkey,
  getSentPaymentsForPeer,
  getReceivedPayments,
  getReceivedPaymentsForSqueak,
  getReceivedPaymentsForPubkey,
  getReceivedPaymentsForPeer,
  getPaymentSummary,
  getPaymentSummaryForSqueak,
  getPaymentSummaryForPubkey,
  getPaymentSummaryForPeer,
} from '../../api/client'

const initialState = {
  sentPaymentsStatus: 'idle',
  sentPayments: [],
  receivedPaymentsStatus: 'idle',
  receivedPayments: [],
  paymentSummary: null,
  paymentSummaryForSqueak: null,
  paymentSummaryForPubkey: null,
  paymentSummaryForPeer: null,
}

// Thunk functions
export const fetchSentPayments = createAsyncThunk(
  'payments/fetchSentPayments',
  async (values) => {
    const response = values.squeakHash ? await getSentPaymentsForSqueak(
      values.squeakHash,
      values.limit,
      values.lastSentPayment,
    )
    : values.pubkey ? await getSentPaymentsForPubkey(
      values.pubkey,
      values.limit,
      values.lastSentPayment,
    )
    : (values.network && values.host && values.port) ? await getSentPaymentsForPeer(
      values.network,
      values.host,
      values.port,
      values.limit,
      values.lastSentPayment,
    )
    : await getSentPayments(
      values.limit,
      values.lastSentPayment,
    );
    return response.getSentPaymentsList();
  }
)

export const fetchReceivedPayments = createAsyncThunk(
  'payments/fetchReceivedPayments',
  async (values) => {
    const response = values.squeakHash ? await getReceivedPaymentsForSqueak(
      values.squeakHash,
      values.limit,
      values.lastReceivedPayment,
    )
    : values.pubkey ? await getReceivedPaymentsForPubkey(
      values.pubkey,
      values.limit,
      values.lastReceivedPayment,
    )
    : (values.network && values.host && values.port) ? await getReceivedPaymentsForPeer(
      values.network,
      values.host,
      values.port,
      values.limit,
      values.lastReceivedPayment,
    )
    : await getReceivedPayments(
      values.limit,
      values.lastReceivedPayment,
    );
    return response.getReceivedPaymentsList();
  }
)

export const fetchPaymentSummary = createAsyncThunk(
  'payments/fetchPaymentSummary',
  async () => {
    console.log('Fetching paymentSummary');
    const response = await getPaymentSummary();
    return response.getPaymentSummary();
  }
)

export const fetchPaymentSummaryForSqueak = createAsyncThunk(
  'payments/fetchPaymentSummaryForSqueak',
  async (values) => {
    console.log('Fetching paymentSummary for squeak:', values.squeakHash);
    const response = await getPaymentSummaryForSqueak(values.squeakHash);
    return response.getPaymentSummary();
  }
)

export const fetchPaymentSummaryForPubkey = createAsyncThunk(
  'payments/fetchPaymentSummaryForPubkey',
  async (values) => {
    console.log('Fetching paymentSummary for pubkey:', values.pubkey);
    const response = await getPaymentSummaryForPubkey(values.pubkey);
    return response.getPaymentSummary();
  }
)

export const fetchPaymentSummaryForPeer = createAsyncThunk(
  'payments/fetchPaymentSummaryForPeer',
  async (values) => {
    console.log('Fetching paymentSummary for peer:', values.network, values.host, values.port);
    const response = await getPaymentSummaryForPeer(values.network, values.host, values.port);
    return response.getPaymentSummary();
  }
)


const paymentsSlice = createSlice({
  name: 'sentPayments',
  initialState,
  reducers: {
    clearSentPayments(state, action) {
      state.sentPaymentsStatus = 'idle'
      state.sentPayments = [];
    },
    clearReceivedPayments(state, action) {
      state.receivedPaymentsStatus = 'idle'
      state.receivedPayments = [];
    },
  },
  extraReducers: (builder) => {
    builder
    .addCase(fetchSentPayments.pending, (state, action) => {
      state.sentPaymentsStatus = 'loading'
    })
    .addCase(fetchSentPayments.fulfilled, (state, action) => {
      const newSentPayments = action.payload;
      state.sentPayments = state.sentPayments.concat(newSentPayments);
      state.sentPaymentsStatus = 'idle'
    })
    .addCase(fetchReceivedPayments.pending, (state, action) => {
      state.receivedPaymentsStatus = 'loading'
    })
    .addCase(fetchReceivedPayments.fulfilled, (state, action) => {
      const newReceivedPayments = action.payload;
      state.receivedPayments = state.receivedPayments.concat(newReceivedPayments);
      state.receivedPaymentsStatus = 'idle'
    })
    .addCase(fetchPaymentSummary.fulfilled, (state, action) => {
      const paymentSummary = action.payload;
      state.paymentSummary = paymentSummary;
    })
    .addCase(fetchPaymentSummaryForSqueak.fulfilled, (state, action) => {
      const paymentSummaryForSqueak = action.payload;
      state.paymentSummaryForSqueak = paymentSummaryForSqueak;
    })
    .addCase(fetchPaymentSummaryForPubkey.fulfilled, (state, action) => {
      const paymentSummaryForPubkey = action.payload;
      state.paymentSummaryForPubkey = paymentSummaryForPubkey;
    })
    .addCase(fetchPaymentSummaryForPeer.fulfilled, (state, action) => {
      const paymentSummaryForPeer = action.payload;
      state.paymentSummaryForPeer = paymentSummaryForPeer;
    })
  },
})

export const {
  clearSentPayments,
  clearReceivedPayments,
} = paymentsSlice.actions

export default paymentsSlice.reducer

export const selectSentPayments = state => state.payments.sentPayments

export const selectLastSentPaymentsSqueak = createSelector(
  selectSentPayments,
  sentPayments => sentPayments.length > 0 && sentPayments[sentPayments.length - 1]
)

export const selectSentPaymentsStatus = state => state.payments.sentPaymentsStatus

export const selectReceivedPayments = state => state.payments.receivedPayments

export const selectLastReceivedPaymentsSqueak = createSelector(
  selectReceivedPayments,
  receivedPayments => receivedPayments.length > 0 && receivedPayments[receivedPayments.length - 1]
)

export const selectReceivedPaymentsStatus = state => state.payments.receivedPaymentsStatus

export const selectPaymentSummary = state => state.payments.paymentSummary

export const selectPaymentSummaryForSqueak = state => state.payments.paymentSummaryForSqueak

export const selectPaymentSummaryForPubkey = state => state.payments.paymentSummaryForPubkey

export const selectPaymentSummaryForPeer = state => state.payments.paymentSummaryForPeer
