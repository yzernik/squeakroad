import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { Link, withRouter } from 'react-router-dom'
import moment from 'moment'

import SqueakCard from '../../components/SqueakCard'
import Loader from '../../components/Loader'


const SentPaymentCard = (props) => {
  const sentPayment = props.sentPayment;
  const dispatch = useDispatch();


  const peerAddress = sentPayment.getPeerAddress();
  const peerUrl = `/app/peer/${peerAddress.getNetwork()}/${peerAddress.getHost()}/${peerAddress.getPort()}`;
  return <div key={sentPayment.getPaymentHash()} className="payment-wapper">
    <div className="search-user-details">
      <div className="search-user-warp">
        <div className="search-user-info">
          <div className="payment-price">
            {sentPayment.getPriceMsat() / 1000} sats
          </div>
          <div className="payment-squeak-hash">
            <b>Squeak Hash</b>:&nbsp;
              <Link style={{color: "blue", fontWeight: 'bold'}} to={`/app/squeak/${sentPayment.getSqueakHash()}`}>
                {sentPayment.getSqueakHash()}
              </Link>
            </div>
            <div className="payment-peer-address">
              <b>Peer</b>:&nbsp;
                <Link to={peerUrl} style={{color: "blue", fontWeight: 'bold'}}>
                  {sentPayment.getPeerAddress().getHost()}:{sentPayment.getPeerAddress().getPort()}
                </Link>
              </div>
              <div className="payment-lightning-node">
                <b>Lightning Node</b>:&nbsp;
                  <a href={`https://amboss.space/node/${sentPayment.getNodePubkey()}`}
                    target="_blank" rel="noopener noreferrer"
                    style={{color: "blue", fontWeight: 'bold'}}
                    >
                    {sentPayment.getNodePubkey()}
                  </a>
                </div>
                <div className="payment-time">{moment(sentPayment.getTimeMs()).format("h:mm A · MMM D, YYYY")}</div>
              </div>
            </div>
          </div>
        </div>

      }

      export default withRouter(SentPaymentCard)
