import { configureStore } from '@reduxjs/toolkit'

import networkReducer from './features/network/networkSlice'
import squeaksReducer from './features/squeaks/squeaksSlice'
import profilesReducer from './features/profiles/profilesSlice'
import peersReducer from './features/peers/peersSlice'
import externalAddressReducer from './features/externalAddress/externalAddressSlice'
import paymentsReducer from './features/payments/paymentsSlice'
import sellPriceReducer from './features/sellPrice/sellPriceSlice'
import accountReducer from './features/account/accountSlice'
import twitterAccounsReducer from './features/twitterAccounts/twitterAccountsSlice'

const store = configureStore({
  reducer: {
    network: networkReducer,
    squeaks: squeaksReducer,
    profiles: profilesReducer,
    peers: peersReducer,
    externalAddress: externalAddressReducer,
    payments: paymentsReducer,
    sellPrice: sellPriceReducer,
    account: accountReducer,
    twitterAccounts: twitterAccounsReducer,
  },
})

export default store
