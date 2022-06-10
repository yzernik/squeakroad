import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { withRouter, Link } from 'react-router-dom'
import moment from 'moment'
import { getPeerImageSrcString } from '../../squeakimages/images';
import { ICON_LAPTOPFILL } from '../../Icons'


import SqueakCard from '../../components/SqueakCard'
import Loader from '../../components/Loader'


import {
  setPeerAutoconnectEnabled,
  setPeerAutoconnectDisabled,
} from './peersSlice'


const PeerCard = (props) => {
  const peer = props.peer;
  const dispatch = useDispatch();

  const enable = (e, id) => {
    e.stopPropagation()
    console.log('Enable peer clicked');
    console.log(id);
    dispatch(setPeerAutoconnectEnabled({peerId: id}));
  }

  const disable = (e,id) => {
    e.stopPropagation()
    console.log('Disable peer clicked');
    console.log(id);
    dispatch(setPeerAutoconnectDisabled({peerId: id}));
  }

  const peerId = peer.getPeerId();
  const peerAddress = peer.getPeerAddress();
  const peerUrl = `/app/peer/${peerAddress.getNetwork()}/${peerAddress.getHost()}/${peerAddress.getPort()}`;
  const savedPeerName = peer.getPeerName();
  const host = peerAddress.getHost();
  const port = peerAddress.getPort();
  const addrStr = host + ':' + port;
  const enabled = peer.getAutoconnect();

  return <Link onClick={(e)=>e.stopPropagation()} to={peerUrl} key={peerId} className="search-result-wapper">
    <div className="search-userPic-wrapper">
      <ICON_LAPTOPFILL styles={{width:'32px', height:"32px"}} />
    </div>
    <div className="search-user-details">
      <div className="search-user-warp">
        <div className="search-user-info">
          <div className="search-user-name">{savedPeerName}</div>
          <div className="search-user-username">{addrStr}</div>
        </div>
        <div onClick={(e)=>{
            e.preventDefault();
            enabled ?
            disable(e, peerId) :
            enable(e, peerId)
          }} className={enabled ? "enable-btn-wrap disable-switch":"enable-btn-wrap"}>
          <span><span>{enabled ? 'Enabled' : 'Disabled'}</span></span>
        </div>
      </div>
      <div className="search-user-bio">
        &nbsp;
      </div>
    </div>
  </Link>
}

export default withRouter(PeerCard)
