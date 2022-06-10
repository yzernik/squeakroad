import {
  createSlice,
  createSelector,
  createAsyncThunk,
  createEntityAdapter,
} from '@reduxjs/toolkit'
import {
  getPeer,
  getPeerByAddress,
  getConnectedPeers,
  connectPeer,
  disconnectPeer,
  getSavedPeers,
  savePeer,
  deletePeer,
  renamePeer,
  enableAutoconnectPeer,
  disableAutoconnectPeer,
} from '../../api/client'

const initialState = {
  currentPeerStatus: 'idle',
  currentPeer: null,
  connectedPeersStatus: 'idle',
  connectedPeers: [],
  savedPeersStatus: 'idle',
  savedPeers: [],
  connectPeerStatus: 'idle',
  disconnectPeerStatus: 'idle',
  savePeerStatus: 'idle',
  deletePeerStatus: 'idle',
}

// Thunk functions
export const fetchPeer = createAsyncThunk(
  'peers/fetchPeer',
  async (values) => {
    console.log('Fetching peer');
    let network = values.network;
    let host = values.host;
    let port = values.port;
    const response = await getPeerByAddress(
      network,
      host,
      port,
    );
    console.log(response);
    return response.getSqueakPeer();
  }
)

export const fetchConnectedPeers = createAsyncThunk(
  'peers/fetchConnectedPeers',
  async () => {
    console.log('Fetching connected peers');
    const response = await getConnectedPeers();
    console.log(response);
    return response.getConnectedPeersList();
  }
)

export const fetchSavedPeers = createAsyncThunk(
  'peers/fetchSavedPeers',
  async () => {
    console.log('Fetching saved peers');
    const response = await getSavedPeers();
    console.log(response);
    return response.getSqueakPeersList();
  }
)

export const setConnectPeer = createAsyncThunk(
  'peers/setConnectPeer',
  async (values) => {
    console.log('Connecting peer');
    let network = values.network;
    let host = values.host;
    let port = values.port;
    await connectPeer(
      network,
      host,
      port,
    );
    const response = await getConnectedPeers();
    return response.getConnectedPeersList();
  }
)

export const setDisconnectPeer = createAsyncThunk(
  'peers/setDisconnectPeer',
  async (values) => {
    console.log('Disconnecting peer');
    let network = values.network;
    let host = values.host;
    let port = values.port;
    await disconnectPeer(
      network,
      host,
      port,
    );
    const response = await getConnectedPeers();
    return response.getConnectedPeersList();
  }
)

export const setSavePeer = createAsyncThunk(
  'peers/setSavePeer',
  async (values) => {
    console.log('Saving peer');
    let name = values.name;
    let network = values.network;
    let host = values.host;
    let port = values.port;
    await savePeer(
      name,
      network,
      host,
      port,
    );
    const response = await getSavedPeers();
    return response.getSqueakPeersList();
  }
)

export const setDeletePeer = createAsyncThunk(
  'peers/setDeletePeer',
  async (values) => {
    console.log('Deleting peer');
    let peerId = values.peerId;
    await deletePeer(peerId);
    const response = await getSavedPeers();
    return response.getSqueakPeersList();
  }
)

export const setPeerAutoconnectEnabled = createAsyncThunk(
  'peers/setPeerAutoconnectEnabled',
  async (values) => {
    console.log('Setting peer autoconnect enabled');
    let peerId = values.peerId;
    await enableAutoconnectPeer(peerId);
    const response = await getSavedPeers();
    return response.getSqueakPeersList();
  }
)

export const setPeerAutoconnectDisabled = createAsyncThunk(
  'peers/setPeerAutoconnectDisabled',
  async (values) => {
    console.log('Setting peer autoconnect disabled');
    let peerId = values.peerId;
    await disableAutoconnectPeer(peerId);
    const response = await getSavedPeers();
    return response.getSqueakPeersList();
  }
)

export const setRenamePeer = createAsyncThunk(
  'profile/setRenamePeer',
  async (values) => {
    console.log('Renaming peer');
    console.log(values.peerId);
    console.log(values.peerName);
    let peerId = values.peerId;
    let peerName = values.peerName;
    await renamePeer(peerId, peerName);
    console.log('Getting renamed peer');
    const response = await getPeer(peerId);
    console.log(response);
    return response.getSqueakPeer();
  }
)


const peersSlice = createSlice({
  name: 'peers',
  initialState,
  reducers: {
    clearAll(state, action) {
      console.log('Clear current peer and status.');
      state.currentPeerStatus = 'idle';
      state.currentPeer = null;
    },
    clearSavedPeers(state, action) {
      state.savedPeersStatus = 'idle'
      state.savedPeers = [];
    },
  },
  extraReducers: (builder) => {
    builder
    .addCase(fetchPeer.pending, (state, action) => {
      state.currentPeerStatus = 'loading'
    })
    .addCase(fetchPeer.fulfilled, (state, action) => {
      const newPeer = action.payload;
      state.currentPeer = newPeer;
      state.currentPeerStatus = 'idle';
    })
    .addCase(fetchConnectedPeers.pending, (state, action) => {
      state.connectedPeersStatus = 'loading'
    })
    .addCase(fetchConnectedPeers.fulfilled, (state, action) => {
      const newConnectedPeers = action.payload;
      state.connectedPeers = newConnectedPeers;
      state.connectedPeersStatus = 'idle';
    })
    .addCase(fetchSavedPeers.pending, (state, action) => {
      state.savedPeersStatus = 'loading'
    })
    .addCase(fetchSavedPeers.fulfilled, (state, action) => {
      const newSavedPeers = action.payload;
      state.savedPeers = newSavedPeers;
      state.savedPeersStatus = 'idle';
    })
    .addCase(setConnectPeer.pending, (state, action) => {
      console.log(action);
      state.connectPeerStatus = 'loading'
    })
    .addCase(setConnectPeer.fulfilled, (state, action) => {
      console.log(action);
      const newConnectedPeers = action.payload;
      state.connectedPeers = newConnectedPeers;
      state.connectPeerStatus = 'idle';
    })
    .addCase(setDisconnectPeer.pending, (state, action) => {
      console.log(action);
      state.disconnectPeerStatus = 'loading'
    })
    .addCase(setDisconnectPeer.fulfilled, (state, action) => {
      console.log(action);
      const newConnectedPeers = action.payload;
      state.connectedPeers = newConnectedPeers;
      state.disconnectPeerStatus = 'idle';
    })
    .addCase(setSavePeer.pending, (state, action) => {
      console.log(action);
      state.savePeerStatus = 'loading'
    })
    .addCase(setSavePeer.fulfilled, (state, action) => {
      console.log(action);
      const newSavedPeers = action.payload;
      const savedAddress = action.meta.arg;
      const newSavedPeer = newSavedPeers.find(savedPeer => {
        return savedPeer.getPeerAddress().getNetwork() === savedAddress.network &&
        savedPeer.getPeerAddress().getHost() === savedAddress.host &&
        savedPeer.getPeerAddress().getPort() == savedAddress.port
      });
      state.currentPeer = newSavedPeer;
      state.savedPeers = newSavedPeers;
      state.savePeerStatus = 'idle';
    })
    .addCase(setDeletePeer.pending, (state, action) => {
      console.log(action);
      state.deletePeerStatus = 'loading'
    })
    .addCase(setDeletePeer.fulfilled, (state, action) => {
      console.log(action);
      const newSavedPeers = action.payload;
      state.currentPeer = null;
      state.savedPeers = newSavedPeers;
      state.deletePeerStatus = 'idle';
    })
    .addCase(setPeerAutoconnectEnabled.fulfilled, (state, action) => {
      console.log(action);
      const newSavedPeers = action.payload;
      const peerId = action.meta.arg.peerId;
      const newSavedPeer = newSavedPeers.find(savedPeer => {
        return savedPeer.getPeerId() === peerId
      });
      state.currentPeer = newSavedPeer;
      state.savedPeers = newSavedPeers;
    })
    .addCase(setPeerAutoconnectDisabled.fulfilled, (state, action) => {
      console.log(action);
      const newSavedPeers = action.payload;
      const peerId = action.meta.arg.peerId;
      const newSavedPeer = newSavedPeers.find(savedPeer => {
        return savedPeer.getPeerId() === peerId
      });
      state.currentPeer = newSavedPeer;
      state.savedPeers = newSavedPeers;
    })
    .addCase(setRenamePeer.pending, (state, action) => {
      state.currentPeerStatus = 'loading'
    })
    .addCase(setRenamePeer.fulfilled, (state, action) => {
      const newPeer = action.payload;
      state.currentPeer = newPeer;
      state.currentPeerStatus = 'idle';
    })
  },
})

export const {
  clearAll,
  clearSavedPeers,
} = peersSlice.actions

export default peersSlice.reducer

export const selectCurrentPeer = state => state.peers.currentPeer

export const selectCurrentPeerStatus = state => state.peers.currentPeerStatus

export const selectSavedPeers = state => state.peers.savedPeers

export const selectSavedPeersStatus = state => state.peers.savedPeersStatus

export const selectConnectedPeers = state => state.peers.connectedPeers

export const selectConnectedPeersStatus = state => state.peers.connectedPeersStatus

export const selectPeerConnectionByAddress = createSelector(
  [
    selectConnectedPeers,
    (state, address) => address
  ],
  (connectedPeers, address) => {
    return connectedPeers.find(obj => {
      return obj.getPeerAddress().getNetwork() === address.network &&
      obj.getPeerAddress().getHost() === address.host &&
      obj.getPeerAddress().getPort() == address.port
    });
  }
)
