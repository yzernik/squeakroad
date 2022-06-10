import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { Link, withRouter } from 'react-router-dom'
import moment from 'moment'

import ReceivedPaymentCard from './ReceivedPaymentCard'
import Loader from '../../components/Loader'


import {
  fetchReceivedPayments,
  clearReceivedPayments,
  selectReceivedPayments,
  selectReceivedPaymentsStatus,
  selectLastReceivedPaymentsSqueak,
} from './paymentsSlice'


const ReceivedPayments = (props) => {
  const receivedPayments = useSelector(selectReceivedPayments);
  const receivedPaymentsStatus = useSelector(selectReceivedPaymentsStatus);
  const lastReceivedPayment = useSelector(selectLastReceivedPaymentsSqueak);
  const dispatch = useDispatch();

  useEffect(() => {
    window.scrollTo(0, 0)
    console.log('fetchReceivedPayments');
    dispatch(clearReceivedPayments());
    dispatch(fetchReceivedPayments({
      squeakHash: props.squeakHash,
      pubkey: props.pubkey,
      network: props.network,
      host: props.host,
      port: props.port,
      limit: 10,
      lastReceivedPayment: null,
    }));
  }, [props.squeakHash])

  const fetchMore = () => {
    dispatch(fetchReceivedPayments({
      squeakHash: props.squeakHash,
      pubkey: props.pubkey,
      network: props.network,
      host: props.host,
      port: props.port,
      limit: 10,
      lastReceivedPayment: lastReceivedPayment,
    }));
  }

  const renderedListItems = receivedPayments.map(receivedPayment=>{
    return <ReceivedPaymentCard receivedPayment={receivedPayment} />
  })

  return <>
  {renderedListItems}

  {receivedPaymentsStatus === 'loading' ?
    <div className="todo-list">
      <Loader />
    </div>
    :
    <div onClick={() => fetchMore()} className='squeak-btn-side squeak-btn-active'>
      LOAD MORE
    </div>
  }

  </>
}

export default withRouter(ReceivedPayments)
