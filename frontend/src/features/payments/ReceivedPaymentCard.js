import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { Link, withRouter } from 'react-router-dom'
import moment from 'moment'

import SqueakCard from '../../components/SqueakCard'
import Loader from '../../components/Loader'



const ReceivedPaymentCard = (props) => {
  const receivedPayment = props.receivedPayment;

  
  return <div key={receivedPayment.getPaymentHash()} className="payment-wapper">
    <div className="search-user-details">
      <div className="search-user-warp">
        <div className="search-user-info">
          <div className="payment-price">
            {receivedPayment.getPriceMsat() / 1000} sats
          </div>
          <div className="payment-squeak-hash">
            <b>Squeak Hash</b>:&nbsp;
              <Link  style={{color: "blue", fontWeight: 'bold'}} to={`/app/squeak/${receivedPayment.getSqueakHash()}`}>{receivedPayment.getSqueakHash()}
              </Link>
            </div>
            <div className="payment-peer-address">
              <b>Peer</b>:&nbsp;
                {receivedPayment.getPeerAddress().getHost()}:{receivedPayment.getPeerAddress().getPort()}
              </div>
              <div className="payment-time">
                {moment(receivedPayment.getTimeMs()).format("h:mm A · MMM D, YYYY")}
              </div>
            </div>
          </div>
        </div>
      </div>

    }

    export default withRouter(ReceivedPaymentCard)
