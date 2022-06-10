import {
  createSlice,
  createSelector,
  createAsyncThunk,
  createEntityAdapter,
} from '@reduxjs/toolkit'
import {
  client,
  getSqueak,
  likeSqueak,
  unlikeSqueak,
  getAncestorSqueaks,
  getReplySqueaks,
  getTimelineSqueaks,
  getSearchSqueaks,
  getProfileSqueaks,
  makeSqueak,
  makeResqueak,
  deleteSqueak,
  getSqueakOffers,
  buySqueak,
  downloadSqueak,
  downloadSqueakSecretKey,
} from '../../api/client'

const initialState = {
  currentSqueakStatus: 'idle',
  currentSqueak: null,
  ancestorSqueaksStatus: 'idle',
  ancestorSqueaks: [],
  replySqueaksStatus: 'idle',
  replySqueaks: [],
  timelineSqueaksStatus: 'idle',
  timelineSqueaks: [],
  searchSqueaksStatus: 'idle',
  searchSqueaks: [],
  profileSqueaksStatus: 'idle',
  profileSqueaks: [],
  makeSqueakStatus: 'idle',
  squeakOffers: [],
  buySqueakStatus: 'idle',
  downloadSqueakStatus: 'idle',
  downloadSqueakOffersStatus: 'idle',
}

// Thunk functions
export const fetchSqueak = createAsyncThunk(
  'squeaks/fetchSqueak',
  async (squeakHash) => {
    console.log('Fetching squeak');
    const response = await getSqueak(squeakHash);
    return response.getSqueakDisplayEntry();
  }
)

export const setLikeSqueak = createAsyncThunk(
  'squeaks/setLikeSqueak',
  async (squeakHash) => {
    console.log('Liking squeak');
    await likeSqueak(squeakHash);
    const response = await getSqueak(squeakHash);
    return response.getSqueakDisplayEntry();
  }
)

export const setUnlikeSqueak = createAsyncThunk(
  'squeaks/setUnlikeSqueak',
  async (squeakHash) => {
    console.log('Unliking squeak');
    await unlikeSqueak(squeakHash);
    const response = await getSqueak(squeakHash);
    return response.getSqueakDisplayEntry();
  }
)

export const setDeleteSqueak = createAsyncThunk(
  'squeaks/setDeleteSqueak',
  async (squeakHash) => {
    console.log('Deleting squeak');
    await deleteSqueak(squeakHash);
    return null;
  }
)

export const fetchAncestorSqueaks = createAsyncThunk(
  'squeaks/fetchAncestorSqueaks',
  async (squeakHash) => {
    console.log('Fetching ancestor squeaks');
    const response = await getAncestorSqueaks(squeakHash);
    return response.getSqueakDisplayEntriesList();
  }
)

export const fetchReplySqueaks = createAsyncThunk(
  'squeaks/fetchReplySqueaks',
  async (values) => {
    console.log('Fetching reply squeaks');
    console.log(values.squeakHash);
    console.log(values.limit);
    console.log(values.lastSqueak);
    const response = await getReplySqueaks(
      values.squeakHash,
      values.limit,
      values.lastSqueak,
    );
    return response.getSqueakDisplayEntriesList();
  }
)

export const fetchTimeline = createAsyncThunk(
  'squeaks/fetchTimeline',
  async (values) => {
    const response = await getTimelineSqueaks(
      values.limit,
      values.lastSqueak,
    );
    return response.getSqueakDisplayEntriesList();
  }
)

export const fetchSearch = createAsyncThunk(
  'searchs/fetchSearch',
  async (values) => {
    const response = await getSearchSqueaks(
      values.searchText,
      values.limit,
      values.lastSqueak,
    );
    return response.getSqueakDisplayEntriesList();
  }
)

export const fetchProfileSqueaks = createAsyncThunk(
  'squeaks/fetchProfileSqueaks',
  async (values) => {
    console.log('Fetching profile squeaks');
    console.log(values.profilePubkey);
    console.log(values.limit);
    console.log(values.lastSqueak);
    const response = await getProfileSqueaks(
      values.profilePubkey,
      values.limit,
      values.lastSqueak,
    );
    return response.getSqueakDisplayEntriesList();
  }
)

export const setMakeSqueak = createAsyncThunk(
  'squeaks/makeSqueak',
  async (values) => {
    console.log('Making squeak');
    let profileId = values.signingProfile;
    let content = values.description;
    let replyTo = values.replyTo;
    let hasRecipient = values.hasRecipient;
    let recipientProfileId = values.recipientProfileId;

    const response = await makeSqueak(
      profileId,
      content,
      replyTo,
      hasRecipient,
      recipientProfileId,
    );
    return response.getSqueakHash();
  }
)

export const setMakeResqueak = createAsyncThunk(
  'squeaks/setMakeResqueak',
  async (values) => {
    console.log('Making resqueak');
    let profileId = values.signingProfile;
    let resqueakedHash = values.resqueakedHash;
    let replyTo = values.replyTo;

    const response = await makeResqueak(
      profileId,
      resqueakedHash,
      replyTo,
    );
    return response.getSqueakHash();
  }
)

export const fetchSqueakOffers = createAsyncThunk(
  'squeaks/fetchSqueakOffers',
  async (squeakHash) => {
    console.log('Fetching squeak offers');
    const response = await getSqueakOffers(squeakHash);
    console.log(response);
    return response.getOffersList();
  }
)

export const setBuySqueak = createAsyncThunk(
  'squeaks/setBuySqueak',
  async (values) => {
    console.log('Buying squeak');
    let offerId = values.offerId;
    let squeakHash = values.squeakHash;
    await buySqueak(offerId);
    const response = await getSqueak(squeakHash);
    return response.getSqueakDisplayEntry();
  }
)

export const setDownloadSqueak = createAsyncThunk(
  'squeaks/setDownloadSqueak',
  async (squeakHash) => {
    console.log('Downloading squeak');
    await downloadSqueak(squeakHash);
    const response = await getSqueak(squeakHash);
    return response.getSqueakDisplayEntry();
  }
)

export const setDownloadSqueakOffers = createAsyncThunk(
  'squeaks/setDownloadSqueakOffers',
  async (squeakHash) => {
    console.log('Downloading squeak offers');
    await downloadSqueakSecretKey(squeakHash);
    const response = await getSqueakOffers(squeakHash);
    return response.getOffersList();
  }
)

const updateCurrentSqueak = (state, newSqueak) => {
  const currentSqueak = state.currentSqueak;
  if (currentSqueak && currentSqueak.getSqueakHash() === newSqueak.getSqueakHash()) {
    state.currentSqueak = newSqueak;
  }

  // Update resqueaked squeak.
  const currentResqueakedSqueak = currentSqueak && currentSqueak.getResqueakedSqueak();
  if (currentResqueakedSqueak && currentResqueakedSqueak.getSqueakHash() === newSqueak.getSqueakHash()) {
    const modifiedSqueak = currentSqueak.clone();
    modifiedSqueak.setResqueakedSqueak(newSqueak);
    state.currentSqueak = modifiedSqueak;
  }
}

const updatedSqueakInArray = (squeakArr, newSqueak) => {
  const currentIndex = squeakArr.findIndex(squeak => squeak.getSqueakHash() === newSqueak.getSqueakHash());
  if (currentIndex != -1) {
    squeakArr[currentIndex] = newSqueak;
  }

  // Update resqueaked squeaks
  for (let i = 0; i < squeakArr.length; i++) {
    const currentSqueak = squeakArr[i];
    const currentResqueakedSqueak = currentSqueak.getResqueakedSqueak();
    if (currentResqueakedSqueak && currentResqueakedSqueak.getSqueakHash() === newSqueak.getSqueakHash()) {
      const modifiedSqueak = currentSqueak.clone();
      modifiedSqueak.setResqueakedSqueak(newSqueak);
      squeakArr[i] = modifiedSqueak;
    }
  }
}

const removeSqueakInArray = (squeakArr, squeakHash) => {
  return squeakArr.filter(squeak => squeak.getSqueakHash() !== squeakHash);

  // Remove resqueaked squeaks
  for (let i = 0; i < squeakArr.length; i++) {
    const currentSqueak = squeakArr[i];
    const currentResqueakedSqueak = currentSqueak.getResqueakedSqueak();
    if (currentResqueakedSqueak && currentResqueakedSqueak.getSqueakHash() === squeakHash) {
      const modifiedSqueak = currentSqueak.clone();
      modifiedSqueak.setResqueakedSqueak(null);
      squeakArr[i] = modifiedSqueak;
    }
  }
}


const squeaksSlice = createSlice({
  name: 'squeaks',
  initialState,
  reducers: {
    clearAll(state, action) {
      console.log('Clear squeak and other data.');
      state.currentSqueakStatus = 'idle';
      state.currentSqueak = null;
    },
    clearAncestors(state, action) {
      console.log('Clear squeak and other data.');
      state.ancestorSqueaksStatus = 'idle';
      state.ancestorSqueaks = []
    },
    clearReplies(state, action) {
      console.log('Clear reply squeaks.');
      state.replySqueaksStatus = 'idle';
      state.replySqueaks = [];
    },
    clearTimeline(state, action) {
      state.timelineSqueaks = [];
    },
    clearSearch(state, action) {
      state.searchSqueaks = [];
    },
    clearProfileSqueaks(state, action) {
      state.profileSqueaks = [];
    },
  },
  extraReducers: (builder) => {
    builder
    .addCase(fetchSqueak.pending, (state, action) => {
      state.currentSqueakStatus = 'loading'
    })
    .addCase(fetchSqueak.fulfilled, (state, action) => {
      console.log(action);
      const newSqueak = action.payload;
      state.currentSqueak = newSqueak;
      state.currentSqueakStatus = 'idle';
    })
    .addCase(setLikeSqueak.fulfilled, (state, action) => {
      console.log(action);
      const newSqueak = action.payload;
      updateCurrentSqueak(state, newSqueak);
      updatedSqueakInArray(state.timelineSqueaks, newSqueak);
      updatedSqueakInArray(state.searchSqueaks, newSqueak);
      updatedSqueakInArray(state.ancestorSqueaks, newSqueak);
      updatedSqueakInArray(state.replySqueaks, newSqueak);
      updatedSqueakInArray(state.profileSqueaks, newSqueak);
    })
    .addCase(setUnlikeSqueak.fulfilled, (state, action) => {
      console.log(action);
      const newSqueak = action.payload;
      updateCurrentSqueak(state, newSqueak);
      updatedSqueakInArray(state.timelineSqueaks, newSqueak);
      updatedSqueakInArray(state.searchSqueaks, newSqueak);
      updatedSqueakInArray(state.ancestorSqueaks, newSqueak);
      updatedSqueakInArray(state.replySqueaks, newSqueak);
      updatedSqueakInArray(state.profileSqueaks, newSqueak);
    })
    .addCase(setDeleteSqueak.fulfilled, (state, action) => {
      console.log(action);
      const deletedSqueakHash = action.meta.arg;
      if (state.currentSqueak && state.currentSqueak.getSqueakHash() === deletedSqueakHash) {
        console.log('Matched current squeak')
        state.currentSqueak = null;
      }
      state.timelineSqueaks = removeSqueakInArray(state.timelineSqueaks, deletedSqueakHash);
      state.searchSqueaks = removeSqueakInArray(state.searchSqueaks, deletedSqueakHash);
      state.ancestorSqueaks = removeSqueakInArray(state.ancestorSqueaks, deletedSqueakHash);
      state.replySqueaks = removeSqueakInArray(state.replySqueaks, deletedSqueakHash);
      state.profileSqueaks = removeSqueakInArray(state.profileSqueaks, deletedSqueakHash);
    })
    .addCase(fetchAncestorSqueaks.pending, (state, action) => {
      state.ancestorSqueaksStatus = 'loading'
    })
    .addCase(fetchAncestorSqueaks.fulfilled, (state, action) => {
      console.log(action);
      const ancestorSqueaks = action.payload;
      state.ancestorSqueaks = ancestorSqueaks;
      state.ancestorSqueaksStatus = 'idle';
    })
    .addCase(fetchReplySqueaks.pending, (state, action) => {
      state.replySqueaksStatus = 'loading'
    })
    .addCase(fetchReplySqueaks.fulfilled, (state, action) => {
      console.log(action);
      const replySqueaks = action.payload;
      state.replySqueaks = replySqueaks;
      state.replySqueaksStatus = 'idle';
    })
    .addCase(fetchTimeline.pending, (state, action) => {
      state.timelineSqueaksStatus = 'loading'
    })
    .addCase(fetchTimeline.fulfilled, (state, action) => {
      const newEntities = action.payload;
      state.timelineSqueaks = state.timelineSqueaks.concat(newEntities);
      state.timelineSqueaksStatus = 'idle'
    })
    .addCase(fetchSearch.pending, (state, action) => {
      state.searchSqueaksStatus = 'loading'
    })
    .addCase(fetchSearch.fulfilled, (state, action) => {
      const newEntities = action.payload;
      state.searchSqueaks = state.searchSqueaks.concat(newEntities);
      state.searchSqueaksStatus = 'idle'
    })
    .addCase(fetchProfileSqueaks.pending, (state, action) => {
      state.profileSqueaksStatus = 'loading'
    })
    .addCase(fetchProfileSqueaks.fulfilled, (state, action) => {
      const newEntities = action.payload;
      state.profileSqueaks = state.profileSqueaks.concat(newEntities);
      state.profileSqueaksStatus = 'idle'
    })
    .addCase(setMakeSqueak.pending, (state, action) => {
      console.log('setMakeSqueak pending');
      state.makeSqueakStatus = 'loading'
    })
    .addCase(setMakeSqueak.rejected, (state, action) => {
      state.makeSqueakStatus = 'idle'
    })
    .addCase(setMakeSqueak.fulfilled, (state, action) => {
      console.log('setMakeSqueak fulfilled');
      console.log(action);
      const newSqueakHash = action.payload;
      state.makeSqueakStatus = 'idle';
      console.log('Go to new squeak');
    })
    .addCase(setMakeResqueak.pending, (state, action) => {
      console.log('setMakeResqueak pending');
      state.makeSqueakStatus = 'loading'
    })
    .addCase(setMakeResqueak.rejected, (state, action) => {
      state.makeSqueakStatus = 'idle'
    })
    .addCase(setMakeResqueak.fulfilled, (state, action) => {
      console.log('setMakeResqueak fulfilled');
      console.log(action);
      const newSqueakHash = action.payload;
      state.makeSqueakStatus = 'idle';
      console.log('Go to new squeak');
    })
    .addCase(fetchSqueakOffers.pending, (state, action) => {
      state.squeakOffers = [];
    })
    .addCase(fetchSqueakOffers.fulfilled, (state, action) => {
      console.log(action);
      const squeakOffers = action.payload;
      state.squeakOffers = squeakOffers;
    })
    .addCase(setBuySqueak.pending, (state, action) => {
      state.buySqueakStatus = 'loading'
    })
    .addCase(setBuySqueak.rejected, (state, action) => {
      state.buySqueakStatus = 'idle'
    })
    .addCase(setBuySqueak.fulfilled, (state, action) => {
      console.log(action);
      state.buySqueakStatus = 'idle';
      const newSqueak = action.payload;
      updateCurrentSqueak(state, newSqueak);
      updatedSqueakInArray(state.timelineSqueaks, newSqueak);
      updatedSqueakInArray(state.searchSqueaks, newSqueak);
      updatedSqueakInArray(state.ancestorSqueaks, newSqueak);
      updatedSqueakInArray(state.replySqueaks, newSqueak);
      updatedSqueakInArray(state.profileSqueaks, newSqueak);
    })
    .addCase(setDownloadSqueak.pending, (state, action) => {
      state.downloadSqueakStatus = 'loading'
    })
    .addCase(setDownloadSqueak.rejected, (state, action) => {
      state.downloadSqueakStatus = 'idle'
    })
    .addCase(setDownloadSqueak.fulfilled, (state, action) => {
      console.log(action);
      state.downloadSqueakStatus = 'idle';
      const newSqueak = action.payload;
      // TODO: check if current squeak is the one that got downloaded.
      state.currentSqueak = newSqueak;
    })
    .addCase(setDownloadSqueakOffers.pending, (state, action) => {
      state.downloadSqueakOffersStatus = 'loading'
    })
    .addCase(setDownloadSqueakOffers.rejected, (state, action) => {
      state.downloadSqueakOffersStatus = 'idle'
    })
    .addCase(setDownloadSqueakOffers.fulfilled, (state, action) => {
      console.log(action);
      state.downloadSqueakOffersStatus = 'idle';
      const squeakOffers = action.payload;
      state.squeakOffers = squeakOffers;
    })
  },
})

export const {
  clearAll,
  clearAncestors,
  clearReplies,
  clearTimeline,
  clearSearch,
  clearProfileSqueaks,
} = squeaksSlice.actions

export default squeaksSlice.reducer

export const selectCurrentSqueak = state => state.squeaks.currentSqueak

export const selectCurrentSqueakStatus = state => state.squeaks.currentSqueakStatus

export const selectAncestorSqueaks = state => state.squeaks.ancestorSqueaks

export const selectAncestorSqueaksStatus = state => state.squeaks.ancestorSqueaksStatus

export const selectReplySqueaks = state => state.squeaks.replySqueaks

export const selectReplySqueaksStatus = state => state.squeaks.replySqueaksStatus

export const selectTimelineSqueaks = state => state.squeaks.timelineSqueaks

export const selectTimelineSqueaksStatus = state => state.squeaks.timelineSqueaksStatus

export const selectLastTimelineSqueak = createSelector(
  selectTimelineSqueaks,
  squeaks => squeaks.length > 0 && squeaks[squeaks.length - 1]
)

export const selectSearchSqueaks = state => state.squeaks.searchSqueaks

export const selectSearchSqueaksStatus = state => state.squeaks.searchSqueaksStatus

export const selectLastSearchSqueak = createSelector(
  selectSearchSqueaks,
  squeaks => squeaks.length > 0 && squeaks[squeaks.length - 1]
)

export const selectProfileSqueaks = state => state.squeaks.profileSqueaks

export const selectProfileSqueaksStatus = state => state.squeaks.profileSqueaksStatus

export const selectLastProfileSqueak = createSelector(
  selectProfileSqueaks,
  squeaks => squeaks.length > 0 && squeaks[squeaks.length - 1]
)

export const selectMakeSqueakStatus = state => state.squeaks.makeSqueakStatus

export const selectSqueakOffers = state => state.squeaks.squeakOffers

export const selectBuySqueakStatus = state => state.squeaks.buySqueakStatus

export const selectDownloadSqueakStatus = state => state.squeaks.downloadSqueakStatus

export const selectDownloadSqueakOffersStatus = state => state.squeaks.downloadSqueakOffersStatus
